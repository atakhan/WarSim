use avian3d::prelude::SpatialQuery;
use bevy::prelude::*;

use super::avian::AvianContactCache;
use super::contact::{BoundaryContactInput, ContactDetection, apply_boundary_contacts};
use super::avian::ContactZoneCollider;
use super::gic::{GicImpulse, GicThrustParams, hero_thrust_impulse_shapecast};
use super::model::{
    Formation, FormationField, FormationHero, FormationMaterial, FormationSide, SoldierSlot,
};
use super::probe::ContactProbeKind;
use super::field::{propagate_pressure_wave, update_material_from_fractures};
use super::settings::LabSettings;

#[derive(Resource, Default)]
pub struct GicLabState {
    pub last_thrust_at: f32,
    pub last_impulse: Option<GicImpulse>,
}

#[allow(clippy::too_many_arguments)]
pub fn run_fmp_layer_1(
    time: Res<Time>,
    settings: Res<LabSettings>,
    avian_cache: Res<AvianContactCache>,
    mut gic_state: ResMut<GicLabState>,
    spatial_query: SpatialQuery,
    heroes: Query<(Entity, &GlobalTransform, &SoldierSlot), With<FormationHero>>,
    formations: Query<&Formation>,
    soldier_slots: Query<(Entity, &SoldierSlot, Option<&ContactZoneCollider>)>,
    mut squads: Query<(&Formation, &mut FormationMaterial, &mut FormationField)>,
) {
    let avian = (settings.contact_probe == ContactProbeKind::Avian)
        .then_some(avian_cache.as_ref());
    if settings.paused {
        return;
    }

    let dt = time.delta_secs().min(1.0 / 20.0);
    let elapsed = time.elapsed_secs();
    let pulse = pressure_pulse(settings.pulse_period, elapsed);
    let mut red_profile = super::field::PressureProfile::Line;
    let mut blue_profile = super::field::PressureProfile::Line;
    let mut red_front = None;
    let mut blue_front = None;
    let mut red_formation = None;
    let mut blue_formation = None;

    for (formation, _, _) in &squads {
        match formation.side {
            FormationSide::Red => {
                red_profile = formation.profile;
                red_front = Some(formation.contact_front());
                red_formation = Some(*formation);
            }
            FormationSide::Blue => {
                blue_profile = formation.profile;
                blue_front = Some(formation.contact_front());
                blue_formation = Some(*formation);
            }
        }
    }

    let mut pending_gic: Option<GicImpulse> = None;
    if settings.gic_enabled
        && elapsed - gic_state.last_thrust_at >= settings.gic_auto_period.max(0.5)
        && let (Ok((hero_entity, hero_tf, hero_slot)), Some(red), Some(blue)) =
            (heroes.single(), red_formation, blue_formation)
        && let Ok(hero_formation) = formations.get(hero_slot.formation)
        && hero_formation.side == FormationSide::Red
        && let Some(impulse) = hero_thrust_impulse_shapecast(
            &spatial_query,
            hero_entity,
            GicThrustParams::from_hero(hero_tf.translation(), &red),
            &blue,
            &soldier_slots,
        )
    {
        gic_state.last_thrust_at = elapsed;
        gic_state.last_impulse = Some(impulse);
        pending_gic = Some(impulse);
    }

    for (formation, mut material, mut field) in &mut squads {
        let Some((incoming_profile, incoming_front)) = (match formation.side {
            FormationSide::Red => blue_front.map(|front| (blue_profile, front)),
            FormationSide::Blue => red_front.map(|front| (red_profile, front)),
        }) else {
            continue;
        };

        let gic_impulse = (formation.side == FormationSide::Blue).then_some(pending_gic).flatten();

        apply_boundary_contacts(
            settings.contact_probe,
            BoundaryContactInput {
                scenario: settings.scenario,
                target_side: formation.side,
                target: formation.contact_front(),
                incoming: incoming_front,
                incoming_profile,
                detection: ContactDetection {
                    contact_distance: settings.contact_distance,
                    base_pressure: settings.impact_strength * pulse,
                },
                dt,
                gic_impulse,
            },
            &mut field,
            avian,
        );

        let mut field_material = material.as_field_material();
        propagate_pressure_wave(field_material, &mut field, settings.field_leak, dt);
        update_material_from_fractures(&mut field_material, &field, dt);
        material.sync_dynamic_state(field_material);
    }
}

fn pressure_pulse(pulse_period: f32, elapsed: f32) -> f32 {
    let phase = (elapsed / pulse_period.max(0.1) * std::f32::consts::TAU).sin();
    0.35 + 0.65 * phase.max(0.0)
}
