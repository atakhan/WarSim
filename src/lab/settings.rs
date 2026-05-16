use bevy::prelude::*;

use super::model::FormationSide;
use super::probe::ContactProbeKind;
use super::scenario::LabScenario;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum FormationMotion {
    #[default]
    Static,
    /// Обе формации сближаются навстречу друг другу.
    ApproachBoth,
    /// Только красный (атакующий) сближается с синим.
    ApproachRed,
    /// Только синий сближается с красным.
    ApproachBlue,
}

impl FormationMotion {
    pub fn is_approaching(self) -> bool {
        !matches!(self, Self::Static)
    }

    pub fn advances(self, side: FormationSide) -> bool {
        match self {
            Self::Static => false,
            Self::ApproachBoth => true,
            Self::ApproachRed => side == FormationSide::Red,
            Self::ApproachBlue => side == FormationSide::Blue,
        }
    }

    pub fn advancing_side_count(self) -> u32 {
        match self {
            Self::Static => 0,
            Self::ApproachBoth => 2,
            Self::ApproachRed | Self::ApproachBlue => 1,
        }
    }
}

#[derive(Resource)]
pub struct LabSettings {
    pub paused: bool,
    pub show_help: bool,
    pub show_legend: bool,
    pub scenario: LabScenario,
    pub impact_strength: f32,
    pub contact_distance: f32,
    pub contact_probe: ContactProbeKind,
    pub formation_motion: FormationMotion,
    pub approach_speed: f32,
    pub field_leak: f32,
    pub pulse_period: f32,
    pub show_fractures: bool,
    pub show_red_stats: bool,
    pub show_blue_stats: bool,
    pub show_organization_visuals: bool,
    /// Скрыть слайдеры подстройки (Guided demo).
    pub lock_tuning: bool,
    pub gic_enabled: bool,
    pub gic_auto_period: f32,
}

impl LabSettings {
    pub fn apply_guided_preset(&mut self) {
        self.scenario = LabScenario::GuidedDemo;
        self.lock_tuning = true;
        self.paused = false;
        self.impact_strength = 9.0;
        self.contact_distance = 5.0;
        self.contact_probe = ContactProbeKind::Avian;
        self.formation_motion = FormationMotion::ApproachRed;
        self.approach_speed = 0.45;
        self.field_leak = 0.08;
        self.pulse_period = 1.4;
        self.show_fractures = true;
        self.show_organization_visuals = true;
        self.show_red_stats = true;
        self.show_blue_stats = true;
        self.show_help = true;
        self.show_legend = true;
        self.gic_enabled = true;
        self.gic_auto_period = 2.5;
    }
}

impl Default for LabSettings {
    fn default() -> Self {
        Self {
            paused: false,
            show_help: true,
            show_legend: true,
            scenario: LabScenario::OffsetContact,
            impact_strength: 9.0,
            contact_distance: 5.0,
            contact_probe: ContactProbeKind::Avian,
            formation_motion: FormationMotion::ApproachRed,
            approach_speed: 0.45,
            field_leak: 0.08,
            pulse_period: 1.4,
            show_fractures: true,
            show_red_stats: true,
            show_blue_stats: true,
            show_organization_visuals: true,
            lock_tuning: false,
            gic_enabled: false,
            gic_auto_period: 2.5,
        }
    }
}
