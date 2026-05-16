use bevy::prelude::*;

use super::model::{Formation, FormationField, FormationMaterial, FormationSide, PressureProfile};

pub type FormationScenarioQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Name,
        &'static mut Formation,
        &'static mut FormationMaterial,
        &'static mut FormationField,
    ),
>;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LabScenario {
    LineVsLine,
    WedgeVsLine,
    OffsetContact,
    FlankPressure,
    LowMoraleDefense,
    FatiguedDefense,
}

impl LabScenario {
    pub const ALL: [Self; 6] = [
        Self::LineVsLine,
        Self::WedgeVsLine,
        Self::OffsetContact,
        Self::FlankPressure,
        Self::LowMoraleDefense,
        Self::FatiguedDefense,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::LineVsLine => "Line vs line",
            Self::WedgeVsLine => "Wedge vs line",
            Self::OffsetContact => "Offset contact",
            Self::FlankPressure => "Flank pressure",
            Self::LowMoraleDefense => "Low morale defense",
            Self::FatiguedDefense => "Fatigued defense",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::LineVsLine => {
                "Базовый контроль: равные линии распределяют давление широким фронтом."
            }
            Self::WedgeVsLine => {
                "Красный клин концентрирует давление в центре синей линии, проверяя пробой yield."
            }
            Self::OffsetContact => {
                "Красный строй смещён к флангу: Contact Zone давит только на перекрытые ряды."
            }
            Self::FlankPressure => {
                "Синяя линия получает фронтальное давление и дополнительную волну с фланга."
            }
            Self::LowMoraleDefense => {
                "Синяя линия держит ту же геометрию, но низкая мораль снижает предел слома."
            }
            Self::FatiguedDefense => {
                "Синяя линия устала: давление хуже гасится, fracture быстрее заражает соседей."
            }
        }
    }

    pub fn watch_for(self) -> &'static str {
        match self {
            Self::LineVsLine => {
                "Смотри на широкое равномерное давление без быстрого каскадного слома."
            }
            Self::WedgeVsLine => {
                "Смотри на концентрацию pressure/fracture в центре обороняющейся линии."
            }
            Self::OffsetContact => {
                "Смотри на частичный вход давления: верхний фланг синей линии должен ломаться раньше нижнего."
            }
            Self::FlankPressure => {
                "Смотри на диагональную волну и ранний fracture у края, где сходятся фронт и фланг."
            }
            Self::LowMoraleDefense => {
                "Смотри, как та же геометрия ломается при меньшем давлении из-за низкого morale."
            }
            Self::FatiguedDefense => {
                "Смотри, как fatigue снижает effective yield и ускоряет распространение fracture."
            }
        }
    }
}

pub fn apply_scenario(scenario: LabScenario, formations: &mut FormationScenarioQuery) {
    for (_, mut formation, mut material, mut field) in formations {
        let (profile, preset) = scenario_preset(scenario, formation.side);
        formation.profile = profile;
        formation.origin = scenario_origin(scenario, formation.side);
        *material = preset;
        field.reset();
    }
}

pub fn scenario_origin(scenario: LabScenario, side: FormationSide) -> Vec3 {
    let lateral_offset = match (scenario, side) {
        (LabScenario::OffsetContact, FormationSide::Red) => 1.55,
        _ => 0.0,
    };

    match side {
        FormationSide::Red => Vec3::new(-2.9, 0.12, lateral_offset),
        FormationSide::Blue => Vec3::new(2.9, 0.12, 0.0),
    }
}

pub fn scenario_preset(
    scenario: LabScenario,
    side: FormationSide,
) -> (PressureProfile, FormationMaterial) {
    match (scenario, side) {
        (LabScenario::LineVsLine, FormationSide::Red) => (
            PressureProfile::Line,
            FormationMaterial {
                stiffness: 6.5,
                forward_multiplier: 1.0,
                lateral_multiplier: 1.0,
                yield_strength: 5.8,
                viscosity: 1.0,
                morale: 0.88,
                fatigue: 0.08,
            },
        ),
        (LabScenario::LineVsLine, FormationSide::Blue) => (
            PressureProfile::Line,
            FormationMaterial {
                stiffness: 6.4,
                forward_multiplier: 1.0,
                lateral_multiplier: 1.0,
                yield_strength: 5.8,
                viscosity: 1.0,
                morale: 0.88,
                fatigue: 0.08,
            },
        ),
        (LabScenario::WedgeVsLine, FormationSide::Red) => (
            PressureProfile::Wedge,
            FormationMaterial {
                stiffness: 7.0,
                forward_multiplier: 1.5,
                lateral_multiplier: 0.5,
                yield_strength: 5.3,
                viscosity: 0.9,
                morale: 0.82,
                fatigue: 0.12,
            },
        ),
        (LabScenario::WedgeVsLine, FormationSide::Blue) => (
            PressureProfile::Line,
            FormationMaterial {
                stiffness: 6.2,
                forward_multiplier: 1.0,
                lateral_multiplier: 1.0,
                yield_strength: 5.8,
                viscosity: 1.1,
                morale: 0.9,
                fatigue: 0.08,
            },
        ),
        (LabScenario::OffsetContact, FormationSide::Red) => (
            PressureProfile::Line,
            FormationMaterial {
                stiffness: 6.5,
                forward_multiplier: 1.0,
                lateral_multiplier: 1.0,
                yield_strength: 5.8,
                viscosity: 1.0,
                morale: 0.86,
                fatigue: 0.08,
            },
        ),
        (LabScenario::OffsetContact, FormationSide::Blue) => (
            PressureProfile::Line,
            FormationMaterial {
                stiffness: 6.1,
                forward_multiplier: 1.0,
                lateral_multiplier: 1.0,
                yield_strength: 5.6,
                viscosity: 1.05,
                morale: 0.84,
                fatigue: 0.1,
            },
        ),
        (LabScenario::FlankPressure, FormationSide::Red) => (
            PressureProfile::Line,
            FormationMaterial {
                stiffness: 6.4,
                forward_multiplier: 1.0,
                lateral_multiplier: 1.0,
                yield_strength: 5.7,
                viscosity: 1.0,
                morale: 0.84,
                fatigue: 0.1,
            },
        ),
        (LabScenario::FlankPressure, FormationSide::Blue) => (
            PressureProfile::Line,
            FormationMaterial {
                stiffness: 5.9,
                forward_multiplier: 1.0,
                lateral_multiplier: 1.0,
                yield_strength: 5.6,
                viscosity: 1.05,
                morale: 0.82,
                fatigue: 0.12,
            },
        ),
        (LabScenario::LowMoraleDefense, FormationSide::Red) => (
            PressureProfile::Line,
            FormationMaterial {
                stiffness: 6.5,
                forward_multiplier: 1.0,
                lateral_multiplier: 1.0,
                yield_strength: 5.8,
                viscosity: 1.0,
                morale: 0.86,
                fatigue: 0.08,
            },
        ),
        (LabScenario::LowMoraleDefense, FormationSide::Blue) => (
            PressureProfile::Line,
            FormationMaterial {
                stiffness: 5.2,
                forward_multiplier: 1.0,
                lateral_multiplier: 1.0,
                yield_strength: 5.4,
                viscosity: 1.25,
                morale: 0.46,
                fatigue: 0.15,
            },
        ),
        (LabScenario::FatiguedDefense, FormationSide::Red) => (
            PressureProfile::Wedge,
            FormationMaterial {
                stiffness: 6.9,
                forward_multiplier: 1.5,
                lateral_multiplier: 0.5,
                yield_strength: 5.6,
                viscosity: 0.95,
                morale: 0.84,
                fatigue: 0.12,
            },
        ),
        (LabScenario::FatiguedDefense, FormationSide::Blue) => (
            PressureProfile::Line,
            FormationMaterial {
                stiffness: 4.8,
                forward_multiplier: 1.0,
                lateral_multiplier: 1.0,
                yield_strength: 5.7,
                viscosity: 1.9,
                morale: 0.78,
                fatigue: 0.62,
            },
        ),
    }
}
