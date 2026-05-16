use bevy::prelude::*;

use super::contact::ContactFront;
use super::field::FieldMaterial;
pub use super::field::{FormationField, PressureProfile};

pub const SLOT_SPACING: f32 = 0.62;

#[derive(Component, Clone, Copy)]
pub struct Formation {
    pub side: FormationSide,
    pub profile: PressureProfile,
    pub origin: Vec3,
    pub forward: Vec3,
    pub columns: usize,
    pub rows: usize,
}

impl Formation {
    pub fn front_column(self) -> usize {
        match self.side {
            FormationSide::Red => self.columns - 1,
            FormationSide::Blue => 0,
        }
    }

    pub fn contact_front(self) -> ContactFront {
        let column_center = (self.columns as f32 - 1.0) * 0.5;
        let half_depth = column_center * SLOT_SPACING;

        ContactFront {
            front_column: self.front_column(),
            rows: self.rows,
            row_spacing: SLOT_SPACING,
            lateral_center: self.origin.z,
            front_position: self.origin.x + self.forward.x.signum() * half_depth,
        }
    }

    #[cfg(test)]
    pub fn slot_position(self, column: usize, row: usize) -> Vec3 {
        let row_center = (self.rows as f32 - 1.0) * 0.5;
        let column_center = (self.columns as f32 - 1.0) * 0.5;
        let row_offset = (row as f32 - row_center) * SLOT_SPACING;
        let column_offset = (column as f32 - column_center) * SLOT_SPACING;
        self.origin + Vec3::new(column_offset, 0.12, row_offset)
    }
}

/// Маркер героя на одном слоте строя (источник GIC, не отдельная боевая петля).
#[derive(Component)]
pub struct FormationHero;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
