use bevy::prelude::*;

use super::scenario::LabScenario;

#[derive(Resource)]
pub struct LabSettings {
    pub paused: bool,
    pub show_help: bool,
    pub show_legend: bool,
    pub scenario: LabScenario,
    pub impact_strength: f32,
    pub field_leak: f32,
    pub pulse_period: f32,
    pub show_fractures: bool,
}

impl Default for LabSettings {
    fn default() -> Self {
        Self {
            paused: false,
            show_help: true,
            show_legend: true,
            scenario: LabScenario::WedgeVsLine,
            impact_strength: 9.0,
            field_leak: 0.08,
            pulse_period: 1.4,
            show_fractures: true,
        }
    }
}
