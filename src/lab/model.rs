use bevy::prelude::*;

use super::field::FieldMaterial;
pub use super::field::{FormationField, PressureProfile};

#[derive(Component, Clone, Copy)]
pub struct Formation {
    pub side: FormationSide,
    pub profile: PressureProfile,
    pub origin: Vec3,
    pub forward: Vec3,
    pub columns: usize,
    pub rows: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FormationSide {
    Red,
    Blue,
}

#[derive(Component, Clone, Copy)]
pub struct FormationMaterial {
    pub stiffness: f32,
    pub forward_multiplier: f32,
    pub lateral_multiplier: f32,
    pub yield_strength: f32,
    pub viscosity: f32,
    pub morale: f32,
    pub fatigue: f32,
}

impl FormationMaterial {
    pub fn as_field_material(self) -> FieldMaterial {
        FieldMaterial {
            stiffness: self.stiffness,
            forward_multiplier: self.forward_multiplier,
            lateral_multiplier: self.lateral_multiplier,
            yield_strength: self.yield_strength,
            viscosity: self.viscosity,
            morale: self.morale,
            fatigue: self.fatigue,
        }
    }

    pub fn sync_dynamic_state(&mut self, material: FieldMaterial) {
        self.morale = material.morale;
        self.fatigue = material.fatigue;
    }

    pub fn effective_yield(&self) -> f32 {
        self.as_field_material().effective_yield()
    }
}

#[derive(Component)]
pub struct SoldierSlot {
    pub formation: Entity,
    pub column: usize,
    pub row: usize,
    pub index: usize,
}

#[derive(Component)]
pub struct SlotVisual {
    pub material: Handle<StandardMaterial>,
}
