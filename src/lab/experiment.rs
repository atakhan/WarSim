use super::field::{
    FieldMaterial, FieldSnapshot, FormationField, inject_flank_pressure, inject_front_pressure,
    propagate_pressure_wave, update_material_from_fractures,
};
use super::model::FormationSide;
use super::scenario::{LabScenario, scenario_preset};

const COLUMNS: usize = 9;
const ROWS: usize = 5;

#[derive(Clone, Copy)]
pub struct ExperimentSettings {
    pub ticks: usize,
    pub dt: f32,
    pub impact_strength: f32,
    pub field_leak: f32,
    pub pulse_period: f32,
}

impl Default for ExperimentSettings {
    fn default() -> Self {
        Self {
            ticks: 160,
            dt: 1.0 / 30.0,
            impact_strength: 9.0,
            field_leak: 0.08,
            pulse_period: 1.4,
        }
    }
}

pub struct ScenarioReport {
    pub red: FormationReport,
    pub blue: FormationReport,
}

pub struct FormationReport {
    pub snapshot: FieldSnapshot,
    pub first_fracture_tick: Option<usize>,
}

struct ExperimentFormation {
    side: FormationSide,
    material: FieldMaterial,
    field: FormationField,
    first_fracture_tick: Option<usize>,
}

pub fn run_scenario_experiment(
    scenario: LabScenario,
    settings: ExperimentSettings,
) -> ScenarioReport {
    let mut red = ExperimentFormation::new(scenario, FormationSide::Red);
    let mut blue = ExperimentFormation::new(scenario, FormationSide::Blue);

    let (red_profile, _) = scenario_preset(scenario, FormationSide::Red);
    let (blue_profile, _) = scenario_preset(scenario, FormationSide::Blue);

    for tick in 0..settings.ticks {
        let elapsed = tick as f32 * settings.dt;
        let pulse = pressure_pulse(settings.pulse_period, elapsed);
        let red_front_column = red.front_column();
        let blue_front_column = blue.front_column();

        inject_front_pressure(
            &mut red.field,
            red_front_column,
            blue_profile,
            settings.impact_strength,
            settings.pulse_period,
            elapsed,
            settings.dt,
        );
        inject_front_pressure(
            &mut blue.field,
            blue_front_column,
            red_profile,
            settings.impact_strength,
            settings.pulse_period,
            elapsed,
            settings.dt,
        );

        if scenario == LabScenario::FlankPressure {
            inject_flank_pressure(
                &mut blue.field,
                settings.impact_strength,
                pulse,
                settings.dt,
            );
        }

        red.step(settings.field_leak, settings.dt, tick);
        blue.step(settings.field_leak, settings.dt, tick);
    }

    ScenarioReport {
        red: red.report(),
        blue: blue.report(),
    }
}

fn pressure_pulse(pulse_period: f32, elapsed: f32) -> f32 {
    let phase = (elapsed / pulse_period.max(0.1) * std::f32::consts::TAU).sin();
    0.35 + 0.65 * phase.max(0.0)
}

impl ExperimentFormation {
    fn new(scenario: LabScenario, side: FormationSide) -> Self {
        let (_, material) = scenario_preset(scenario, side);
        Self {
            side,
            material: material.as_field_material(),
            field: FormationField::new(COLUMNS, ROWS),
            first_fracture_tick: None,
        }
    }

    fn front_column(&self) -> usize {
        match self.side {
            FormationSide::Red => self.field.width - 1,
            FormationSide::Blue => 0,
        }
    }

    fn step(&mut self, field_leak: f32, dt: f32, tick: usize) {
        propagate_pressure_wave(self.material, &mut self.field, field_leak, dt);
        update_material_from_fractures(&mut self.material, &self.field, dt);

        if self.first_fracture_tick.is_none()
            && self.field.fractured.iter().any(|fractured| *fractured)
        {
            self.first_fracture_tick = Some(tick);
        }
    }

    fn report(self) -> FormationReport {
        FormationReport {
            snapshot: self.field.snapshot(),
            first_fracture_tick: self.first_fracture_tick,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wedge_vs_line_concentrates_blue_damage_toward_center() {
        let report = run_scenario_experiment(
            LabScenario::WedgeVsLine,
            ExperimentSettings {
                ticks: 180,
                ..ExperimentSettings::default()
            },
        );

        assert!(
            report.red.snapshot.front_fracture_ratio >= report.red.snapshot.rear_fracture_ratio
        );
        assert!(report.blue.snapshot.center_pressure > report.blue.snapshot.edge_pressure);
        assert!(
            report.blue.snapshot.center_fracture_ratio >= report.blue.snapshot.edge_fracture_ratio
        );
    }

    #[test]
    fn flank_pressure_creates_blue_flank_asymmetry() {
        let flank = run_scenario_experiment(
            LabScenario::FlankPressure,
            ExperimentSettings {
                ticks: 180,
                ..ExperimentSettings::default()
            },
        );
        let line = run_scenario_experiment(
            LabScenario::LineVsLine,
            ExperimentSettings {
                ticks: 180,
                ..ExperimentSettings::default()
            },
        );

        assert!(
            flank.blue.snapshot.flank_fracture_asymmetry()
                > line.blue.snapshot.flank_fracture_asymmetry()
        );
        assert!(
            flank.blue.snapshot.upper_flank_fracture_ratio
                > flank.blue.snapshot.lower_flank_fracture_ratio
        );
    }

    #[test]
    fn low_morale_defense_breaks_before_normal_line() {
        let low_morale = run_scenario_experiment(
            LabScenario::LowMoraleDefense,
            ExperimentSettings {
                ticks: 180,
                ..ExperimentSettings::default()
            },
        );
        let normal = run_scenario_experiment(
            LabScenario::LineVsLine,
            ExperimentSettings {
                ticks: 180,
                ..ExperimentSettings::default()
            },
        );

        assert!(low_morale.blue.first_fracture_tick.is_some());
        assert!(
            low_morale.blue.first_fracture_tick.unwrap_or(usize::MAX)
                < normal.blue.first_fracture_tick.unwrap_or(usize::MAX)
        );
        assert!(
            low_morale.blue.snapshot.center_fracture_ratio
                >= normal.blue.snapshot.center_fracture_ratio
        );
    }
}
