use bevy::prelude::*;

use super::field::{
    inject_flank_pressure, inject_front_pressure, propagate_pressure_wave,
    update_material_from_fractures,
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

    for (formation, _, _) in &formations {
        match formation.side {
            FormationSide::Red => red_profile = formation.profile,
            FormationSide::Blue => blue_profile = formation.profile,
        }
    }

    for (formation, mut material, mut field) in &mut formations {
        let incoming_profile = match formation.side {
            FormationSide::Red => blue_profile,
            FormationSide::Blue => red_profile,
        };

        inject_boundary_pressure(
            settings.scenario,
            formation,
            incoming_profile,
            &mut field,
            BoundaryPressure {
                impact_strength: settings.impact_strength,
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
    pulse_period: f32,
    elapsed: f32,
    dt: f32,
}

fn inject_boundary_pressure(
    scenario: LabScenario,
    formation: &Formation,
    incoming_profile: PressureProfile,
    field: &mut FormationField,
    pressure: BoundaryPressure,
) {
    let front_column = match formation.side {
        FormationSide::Red => field.width - 1,
        FormationSide::Blue => 0,
    };

    let phase = (pressure.elapsed / pressure.pulse_period.max(0.1) * std::f32::consts::TAU).sin();
    let pulse = 0.35 + 0.65 * phase.max(0.0);

    inject_front_pressure(
        field,
        front_column,
        incoming_profile,
        pressure.impact_strength,
        pressure.pulse_period,
        pressure.elapsed,
        pressure.dt,
    );

    if scenario == LabScenario::FlankPressure && formation.side == FormationSide::Blue {
        inject_flank_pressure(field, pressure.impact_strength, pulse, pressure.dt);
    }
}
