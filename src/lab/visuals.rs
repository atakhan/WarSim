use bevy::prelude::*;

use super::model::{
    Formation, FormationField, FormationSide, SLOT_SPACING, SlotVisual, SoldierSlot,
};
use super::settings::LabSettings;

pub fn update_soldier_visuals(
    settings: Res<LabSettings>,
    formations: Query<(&Formation, &FormationField)>,
    mut soldiers: Query<(&SoldierSlot, &SlotVisual, &mut Transform)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (slot, visual, mut transform) in &mut soldiers {
        let Ok((formation, field)) = formations.get(slot.formation) else {
            continue;
        };

        let pressure = field.pressure[slot.index];
        let fractured = field.fractured[slot.index];
        let row_center = (formation.rows as f32 - 1.0) * 0.5;
        let column_center = (formation.columns as f32 - 1.0) * 0.5;
        let row_offset = (slot.row as f32 - row_center) * SLOT_SPACING;
        let column_offset = (slot.column as f32 - column_center) * SLOT_SPACING;

        let row_sign = if slot.row as f32 >= row_center {
            1.0
        } else {
            -1.0
        };
        let fracture_amount = if fractured { 1.0 } else { 0.0 };
        let pressure_amount = pressure.clamp(0.0, 10.0);

        let pressure_displacement = -formation.forward * pressure_amount * 0.075;
        let fracture_displacement =
            (-formation.forward * 0.38 + Vec3::Z * row_sign * 0.34) * fracture_amount;
        let height = 0.12 + pressure_amount * 0.02 - fracture_amount * 0.035;

        transform.translation = formation.origin
            + Vec3::new(column_offset, height, row_offset)
            + pressure_displacement
            + fracture_displacement;

        transform.scale = if fractured {
            Vec3::new(1.18, 0.42, 1.18)
        } else {
            Vec3::new(1.0, 1.0 + pressure_amount * 0.035, 1.0)
        };
        transform.rotation = if fractured {
            Quat::from_rotation_z(row_sign * 0.28)
        } else {
            Quat::IDENTITY
        };

        if let Some(material) = materials.get_mut(&visual.material) {
            let normalized_pressure = (pressure_amount / 8.0).clamp(0.0, 1.0);
            material.base_color = side_color(
                formation.side,
                normalized_pressure,
                settings.show_fractures && fractured,
            );
        }
    }
}

pub fn side_color(side: FormationSide, pressure: f32, fractured: bool) -> Color {
    if fractured {
        return Color::srgb(1.0, 0.78, 0.18);
    }

    match side {
        FormationSide::Red => Color::srgb(0.42 + pressure * 0.58, 0.08 + pressure * 0.3, 0.08),
        FormationSide::Blue => Color::srgb(0.08, 0.16 + pressure * 0.35, 0.45 + pressure * 0.55),
    }
}
