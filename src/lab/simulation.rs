use bevy::prelude::*;

use super::contact::{BoundaryContactInput, ContactDetection, apply_boundary_contacts};
use super::field::{propagate_pressure_wave, update_material_from_fractures};
use super::model::{Formation, FormationField, FormationMaterial, FormationSide, PressureProfile};
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
    let pulse = pressure_pulse(settings.pulse_period, elapsed);
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

        apply_boundary_contacts(
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
            },
            &mut field,
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
