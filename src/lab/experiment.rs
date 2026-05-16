use bevy::prelude::Vec3;

use super::contact::{
    BoundaryContactInput, ContactDetection, ContactFront, apply_boundary_contacts,
};
use super::probe::ContactProbeKind;
use super::field::{
    FieldMaterial, FieldSnapshot, FormationField, propagate_pressure_wave,
    update_material_from_fractures,
};
use super::model::{Formation, FormationSide, SLOT_SPACING};
use super::motion::{FrontPositions, should_advance_approach};
use super::scenario::{LabScenario, scenario_origin, scenario_preset};
use super::gic::{
    DefenderFrontSample, GicImpulse, GicThrustParams, compute_thrust_impulse_geometric,
};
use super::settings::FormationMotion;

const COLUMNS: usize = 9;
const ROWS: usize = 5;

#[derive(Clone, Copy)]
pub struct ExperimentSettings {
    pub ticks: usize,
    pub dt: f32,
    pub impact_strength: f32,
    pub contact_distance: f32,
    pub formation_motion: FormationMotion,
    pub approach_speed: f32,
    pub field_leak: f32,
    pub pulse_period: f32,
    /// Если задан — один thrust GIC на этом тике (для regression).
    pub gic_thrust_tick: Option<usize>,
}

impl Default for ExperimentSettings {
    fn default() -> Self {
        Self {
            ticks: 160,
            dt: 1.0 / 30.0,
            impact_strength: 9.0,
            contact_distance: 5.0,
            formation_motion: FormationMotion::Static,
            approach_speed: 0.35,
            field_leak: 0.08,
            pulse_period: 1.4,
            gic_thrust_tick: None,
        }
    }
}

impl ExperimentSettings {
    fn build_gic_impulse(
        &self,
        red: &ExperimentFormation,
        blue: &ExperimentFormation,
    ) -> Option<GicImpulse> {
        let hero_row = ROWS / 2;
        let hero = red.slot_position(red.front_column(), hero_row);
        let thrust = GicThrustParams {
            origin: hero,
            direction: Vec3::X,
            length: 2.8,
            weapon_mass: 1.2,
            tip_velocity: 4.0,
        };
        let samples = blue.front_row_samples();
        let defender = Formation {
            side: FormationSide::Blue,
            profile: super::field::PressureProfile::Line,
            origin: Vec3::new(blue.origin_x, 0.12, blue.lateral_center),
            forward: Vec3::NEG_X,
            columns: COLUMNS,
            rows: ROWS,
        };
        compute_thrust_impulse_geometric(thrust, &samples, &defender)
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
    origin_x: f32,
    lateral_center: f32,
    forward_x: f32,
    first_fracture_tick: Option<usize>,
}

fn advance_approach(
    red: &mut ExperimentFormation,
    blue: &mut ExperimentFormation,
    settings: &ExperimentSettings,
    dt: f32,
) {
    let motion = settings.formation_motion;
    if !motion.is_approaching() {
        return;
    }

    let fronts = FrontPositions {
        red: red.front_position(),
        blue: blue.front_position(),
    };

    if !should_advance_approach(
        fronts,
        settings.approach_speed,
        dt,
        motion.advancing_side_count(),
    ) {
        return;
    }

    if motion.advances(FormationSide::Red) {
        red.origin_x += red.forward_x * settings.approach_speed * dt;
    }
    if motion.advances(FormationSide::Blue) {
        blue.origin_x += blue.forward_x * settings.approach_speed * dt;
    }
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
        advance_approach(&mut red, &mut blue, &settings, settings.dt);

        let red_front_column = red.front_column();
        let blue_front_column = blue.front_column();

        let detection = ContactDetection {
            contact_distance: settings.contact_distance,
            base_pressure: settings.impact_strength * pulse,
        };

        apply_boundary_contacts(
            ContactProbeKind::Synthetic,
            BoundaryContactInput {
                scenario,
                target_side: FormationSide::Red,
                target: red.contact_front(red_front_column),
                incoming: blue.contact_front(blue_front_column),
                incoming_profile: blue_profile,
                detection,
                dt: settings.dt,
                gic_impulse: None,
            },
            &mut red.field,
            None,
        );

        let gic_impulse = settings
            .gic_thrust_tick
            .filter(|&t| t == tick)
            .and_then(|_| settings.build_gic_impulse(&red, &blue));

        apply_boundary_contacts(
            ContactProbeKind::Synthetic,
            BoundaryContactInput {
                scenario,
                target_side: FormationSide::Blue,
                target: blue.contact_front(blue_front_column),
                incoming: red.contact_front(red_front_column),
                incoming_profile: red_profile,
                detection,
                dt: settings.dt,
                gic_impulse,
            },
            &mut blue.field,
            None,
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
            origin_x: origin.x,
            lateral_center: origin.z,
            forward_x: match side {
                FormationSide::Red => 1.0,
                FormationSide::Blue => -1.0,
            },
            first_fracture_tick: None,
        }
    }

    fn front_position(&self) -> f32 {
        front_position(self.side, self.origin_x)
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
            front_position: self.front_position(),
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
            snapshot: self.field.snapshot(self.front_column()),
            first_fracture_tick: self.first_fracture_tick,
        }
    }

    fn slot_position(&self, column: usize, row: usize) -> Vec3 {
        let row_center = (ROWS as f32 - 1.0) * 0.5;
        let column_center = (COLUMNS as f32 - 1.0) * 0.5;
        let row_offset = (row as f32 - row_center) * SLOT_SPACING;
        let column_offset = (column as f32 - column_center) * SLOT_SPACING;
        Vec3::new(
            self.origin_x + column_offset,
            0.12,
            self.lateral_center + row_offset,
        )
    }

    fn front_row_samples(&self) -> Vec<DefenderFrontSample> {
        let column = self.front_column();
        (0..self.field.height)
            .map(|row| DefenderFrontSample {
                row,
                position: self.slot_position(column, row),
            })
            .collect()
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
    fn partial_overlap_erodes_front_organization_more_than_full_contact() {
        let offset = run_scenario_experiment(
            LabScenario::OffsetContact,
            ExperimentSettings {
                ticks: 120,
                ..ExperimentSettings::default()
            },
        );
        let line = run_scenario_experiment(
            LabScenario::LineVsLine,
            ExperimentSettings {
                ticks: 120,
                ..ExperimentSettings::default()
            },
        );

        assert!(offset.blue.snapshot.front_organization_min < 1.0);
        assert!(
            offset.blue.snapshot.front_organization_min
                < line.blue.snapshot.front_organization_min
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
        let line = run_scenario_experiment(
            LabScenario::LineVsLine,
            ExperimentSettings {
                ticks: 180,
                ..ExperimentSettings::default()
            },
        );

        assert!(
            offset.blue.snapshot.flank_pressure_asymmetry()
                > line.blue.snapshot.flank_pressure_asymmetry()
        );
        assert!(
            offset.blue.snapshot.upper_flank_fracture_ratio
                .max(offset.blue.snapshot.lower_flank_fracture_ratio)
                >= line.blue.snapshot.edge_fracture_ratio
        );
    }

    #[test]
    fn red_only_approach_increases_defender_pressure_over_static() {
        let static_run = run_scenario_experiment(
            LabScenario::WedgeVsLine,
            ExperimentSettings {
                ticks: 160,
                ..ExperimentSettings::default()
            },
        );
        let red_approach = run_scenario_experiment(
            LabScenario::WedgeVsLine,
            ExperimentSettings {
                ticks: 160,
                formation_motion: FormationMotion::ApproachRed,
                approach_speed: 0.5,
                ..ExperimentSettings::default()
            },
        );

        assert!(
            red_approach.blue.snapshot.center_pressure
                > static_run.blue.snapshot.center_pressure
        );
    }

    #[test]
    fn mutual_approach_reaches_higher_pressure_than_red_only_at_same_ticks() {
        let red_only = run_scenario_experiment(
            LabScenario::LineVsLine,
            ExperimentSettings {
                ticks: 120,
                formation_motion: FormationMotion::ApproachRed,
                approach_speed: 0.45,
                ..ExperimentSettings::default()
            },
        );
        let mutual = run_scenario_experiment(
            LabScenario::LineVsLine,
            ExperimentSettings {
                ticks: 120,
                formation_motion: FormationMotion::ApproachBoth,
                approach_speed: 0.45,
                ..ExperimentSettings::default()
            },
        );

        assert!(
            mutual.blue.snapshot.center_pressure > red_only.blue.snapshot.center_pressure
        );
    }

    #[test]
    fn approach_increases_blue_peak_pressure_over_static_line() {
        let static_run = run_scenario_experiment(
            LabScenario::LineVsLine,
            ExperimentSettings {
                ticks: 140,
                ..ExperimentSettings::default()
            },
        );
        let approach = run_scenario_experiment(
            LabScenario::LineVsLine,
            ExperimentSettings {
                ticks: 140,
                formation_motion: FormationMotion::ApproachBoth,
                approach_speed: 0.45,
                ..ExperimentSettings::default()
            },
        );

        assert!(approach.blue.snapshot.center_pressure > static_run.blue.snapshot.center_pressure);
    }

    #[test]
    fn wedge_vs_phalanx_holds_center_better_than_wedge_vs_line() {
        let settings = ExperimentSettings {
            ticks: 320,
            impact_strength: 11.5,
            formation_motion: FormationMotion::ApproachRed,
            approach_speed: 0.5,
            ..ExperimentSettings::default()
        };
        let phalanx = run_scenario_experiment(LabScenario::WedgeVsPhalanx, settings);
        let line = run_scenario_experiment(LabScenario::WedgeVsLine, settings);

        assert!(
            line.blue.snapshot.center_fracture_ratio
                >= line.blue.snapshot.edge_fracture_ratio
        );
        assert!(line.blue.first_fracture_tick.is_some());
        assert!(
            phalanx.blue.snapshot.center_fracture_ratio
                <= line.blue.snapshot.center_fracture_ratio
        );
        assert!(
            phalanx.blue.first_fracture_tick.unwrap_or(usize::MAX)
                > line.blue.first_fracture_tick.unwrap_or(0)
                || phalanx.blue.snapshot.center_fracture_ratio
                    < line.blue.snapshot.center_fracture_ratio
        );
    }

    #[test]
    fn phalanx_vs_crowd_crowd_fractures_faster_than_line_defense() {
        let phalanx_crowd = run_scenario_experiment(
            LabScenario::PhalanxVsCrowd,
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

        assert!(phalanx_crowd.blue.first_fracture_tick.is_some());
        assert!(
            phalanx_crowd.blue.first_fracture_tick.unwrap_or(usize::MAX)
                < line.blue.first_fracture_tick.unwrap_or(usize::MAX)
        );
        assert!(
            phalanx_crowd.blue.snapshot.center_fracture_ratio
                > line.blue.snapshot.center_fracture_ratio
        );
        assert!(
            phalanx_crowd.blue.snapshot.edge_pressure
                > phalanx_crowd.blue.snapshot.center_pressure * 0.55
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

    #[test]
    fn gic_thrust_applies_when_formations_are_engaged() {
        let settings = ExperimentSettings {
            ticks: 80,
            formation_motion: FormationMotion::ApproachRed,
            approach_speed: 0.5,
            ..ExperimentSettings::default()
        };
        let mut red = ExperimentFormation::new(LabScenario::LineVsLine, FormationSide::Red);
        let mut blue = ExperimentFormation::new(LabScenario::LineVsLine, FormationSide::Blue);

        for _ in 0..75 {
            advance_approach(&mut red, &mut blue, &settings, settings.dt);
        }

        let impulse = settings.build_gic_impulse(&red, &blue).expect("engaged thrust");
        assert!(impulse.pressure_boost > 0.0);
        assert!(!impulse.row_range.is_empty());
    }
}
