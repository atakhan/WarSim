use bevy::prelude::*;

use super::contact::{ContactDetection, ContactFront, detect_contact_request};
use super::field::{
    inject_flank_pressure, propagate_pressure_wave, update_material_from_fractures,
};
use super::model::{Formation, FormationField, FormationMaterial, FormationSide, PressureProfile};
use super::scenario::LabScenario;
use super::settings::LabSettings;

pub fn run_fmp_layer_1(
    time: Res<Time>,
    settings: Res<LabSettings>,
    mut formations: Query<(&Formation, &mut FormationMaterial, &mut FormationField)>,
) {
    if settings.paused {
        return;
    }

    let dt = time.delta_secs().min(1.0 / 20.0);
    let elapsed = time.elapsed_secs();
    let mut red_profile = PressureProfile::Line;
    let mut blue_profile = PressureProfile::Line;
    let mut red_front = None;
    let mut blue_front = None;

    for (formation, _, _) in &formations {
        match formation.side {
            FormationSide::Red => {
                red_profile = formation.profile;
                red_front = Some(formation.contact_front());
            }
            FormationSide::Blue => {
                blue_profile = formation.profile;
                blue_front = Some(formation.contact_front());
            }
        }
    }

    for (formation, mut material, mut field) in &mut formations {
        let Some((incoming_profile, incoming_front)) = (match formation.side {
            FormationSide::Red => blue_front.map(|front| (blue_profile, front)),
            FormationSide::Blue => red_front.map(|front| (red_profile, front)),
        }) else {
            continue;
        };

        inject_boundary_pressure(
            settings.scenario,
            formation,
            incoming_front,
            incoming_profile,
            &mut field,
            BoundaryPressure {
                impact_strength: settings.impact_strength,
                contact_distance: 5.0,
                pulse_period: settings.pulse_period,
                elapsed,
                dt,
            },
        );

        let mut field_material = material.as_field_material();
        propagate_pressure_wave(field_material, &mut field, settings.field_leak, dt);
        update_material_from_fractures(&mut field_material, &field, dt);
        material.sync_dynamic_state(field_material);
    }
}

#[derive(Clone, Copy)]
struct BoundaryPressure {
    impact_strength: f32,
    contact_distance: f32,
    pulse_period: f32,
    elapsed: f32,
    dt: f32,
}

fn inject_boundary_pressure(
    scenario: LabScenario,
    formation: &Formation,
    incoming_front: ContactFront,
    incoming_profile: PressureProfile,
    field: &mut FormationField,
    pressure: BoundaryPressure,
) {
    let phase = (pressure.elapsed / pressure.pulse_period.max(0.1) * std::f32::consts::TAU).sin();
    let pulse = 0.35 + 0.65 * phase.max(0.0);

    if let Some(contact) = detect_contact_request(
        formation.contact_front(),
        incoming_front,
        incoming_profile,
        ContactDetection {
            contact_distance: pressure.contact_distance,
            base_pressure: pressure.impact_strength * pulse,
        },
    ) {
        contact.resolve().apply_to_field(field, pressure.dt);
    }

    if scenario == LabScenario::FlankPressure && formation.side == FormationSide::Blue {
        inject_flank_pressure(field, pressure.impact_strength, pulse, pressure.dt);
    }
}
