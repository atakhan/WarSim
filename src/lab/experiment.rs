use super::contact::{
    BoundaryContactInput, ContactDetection, ContactFront, apply_boundary_contacts,
};
use super::field::{
    FieldMaterial, FieldSnapshot, FormationField, propagate_pressure_wave,
    update_material_from_fractures,
};
use super::model::{FormationSide, SLOT_SPACING};
use super::scenario::{LabScenario, scenario_origin, scenario_preset};

const COLUMNS: usize = 9;
const ROWS: usize = 5;

#[derive(Clone, Copy)]
pub struct ExperimentSettings {
    pub ticks: usize,
    pub dt: f32,
    pub impact_strength: f32,
    pub contact_distance: f32,
    pub field_leak: f32,
    pub pulse_period: f32,
}

impl Default for ExperimentSettings {
    fn default() -> Self {
        Self {
            ticks: 160,
            dt: 1.0 / 30.0,
            impact_strength: 9.0,
            contact_distance: 5.0,
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
    lateral_center: f32,
    front_position: f32,
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

        let detection = ContactDetection {
            contact_distance: settings.contact_distance,
            base_pressure: settings.impact_strength * pulse,
        };

        apply_boundary_contacts(
            BoundaryContactInput {
                scenario,
                target_side: FormationSide::Red,
                target: red.contact_front(red_front_column),
                incoming: blue.contact_front(blue_front_column),
                incoming_profile: blue_profile,
                detection,
                dt: settings.dt,
            },
            &mut red.field,
        );
        apply_boundary_contacts(
            BoundaryContactInput {
                scenario,
                target_side: FormationSide::Blue,
                target: blue.contact_front(blue_front_column),
                incoming: red.contact_front(red_front_column),
                incoming_profile: red_profile,
                detection,
                dt: settings.dt,
            },
            &mut blue.field,
        );

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
        let origin = scenario_origin(scenario, side);
        Self {
            side,
            material: material.as_field_material(),
            field: FormationField::new(COLUMNS, ROWS),
            lateral_center: origin.z,
            front_position: front_position(side, origin.x),
            first_fracture_tick: None,
        }
    }

    fn front_column(&self) -> usize {
        match self.side {
            FormationSide::Red => self.field.width - 1,
            FormationSide::Blue => 0,
        }
    }

    fn contact_front(&self, front_column: usize) -> ContactFront {
        ContactFront {
            front_column,
            rows: self.field.height,
            row_spacing: SLOT_SPACING,
            lateral_center: self.lateral_center,
            front_position: self.front_position,
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

fn front_position(side: FormationSide, origin_x: f32) -> f32 {
    let column_center = (COLUMNS as f32 - 1.0) * 0.5;
    let half_depth = column_center * SLOT_SPACING;

    match side {
        FormationSide::Red => origin_x + half_depth,
        FormationSide::Blue => origin_x - half_depth,
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
    fn offset_contact_limits_blue_damage_to_overlapped_flank() {
        let offset = run_scenario_experiment(
            LabScenario::OffsetContact,
            ExperimentSettings {
                ticks: 180,
                ..ExperimentSettings::default()
            },
        );

        assert!(
            offset.blue.snapshot.upper_flank_pressure > offset.blue.snapshot.lower_flank_pressure
        );
        assert!(offset.blue.snapshot.flank_pressure_asymmetry() > 0.0);
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
