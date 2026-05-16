use bevy::prelude::*;

use super::model::{
    Formation, FormationField, FormationHero, FormationSide, SLOT_SPACING, SlotVisual,
    SoldierSlot,
};
use super::settings::LabSettings;

pub fn update_soldier_visuals(
    settings: Res<LabSettings>,
    formations: Query<(&Formation, &FormationField)>,
    mut soldiers: Query<(
        &SoldierSlot,
        &SlotVisual,
        &mut Transform,
        Option<&FormationHero>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (slot, visual, mut transform, hero) in &mut soldiers {
        let Ok((formation, field)) = formations.get(slot.formation) else {
            continue;
        };

        let pressure = field.pressure[slot.index];
        let fractured = field.fractured[slot.index];
        let organization = field.organization[slot.index];
        let on_front = slot.column == formation.front_column();
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
        let pressure_amount = pressure.clamp(0.0, 12.0);
        let org_loss = (1.0 - organization).clamp(0.0, 1.0);
        let front_org_loss = if on_front { org_loss } else { org_loss * 0.35 };

        let pressure_displacement = -formation.forward * pressure_amount * 0.09;
        let fracture_displacement =
            (-formation.forward * 0.38 + Vec3::Z * row_sign * 0.34) * fracture_amount;
        let disorganization_spread =
            Vec3::Z * row_sign * front_org_loss * 0.28 * (1.0 - fracture_amount);
        let height = 0.12 + pressure_amount * 0.025 - fracture_amount * 0.035 - front_org_loss * 0.04;

        transform.translation = formation.origin
            + Vec3::new(column_offset, height, row_offset)
            + pressure_displacement
            + fracture_displacement
            + disorganization_spread;

        transform.scale = if fractured {
            Vec3::new(1.18, 0.42, 1.18)
        } else {
            let swell = pressure_amount * 0.04 + (1.0 - front_org_loss) * 0.02;
            Vec3::new(1.0 - front_org_loss * 0.08, 1.0 + swell, 1.0 - front_org_loss * 0.08)
        };
        transform.rotation = if fractured {
            Quat::from_rotation_z(row_sign * 0.28)
        } else {
            Quat::from_rotation_z(row_sign * front_org_loss * 0.12)
        };

        if let Some(material) = materials.get_mut(&visual.material) {
            let normalized_pressure = (pressure_amount / 6.0).clamp(0.0, 1.0);
            let mut color = side_color(
                formation.side,
                normalized_pressure,
                settings.show_fractures && fractured,
            );
            if settings.show_organization_visuals && !fractured {
                color = organization_tint(color, organization);
            }
            if on_front && pressure_amount > 1.5 && !fractured {
                color = color.mix(
                    &Color::srgb(0.95, 0.95, 1.0),
                    (normalized_pressure * 0.18).clamp(0.0, 0.22),
                );
            }
            if hero.is_some() {
                color = color.mix(&Color::srgb(1.0, 0.88, 0.35), 0.45);
            }
            material.base_color = color;
        }
    }
}

fn organization_tint(base: Color, organization: f32) -> Color {
    let strain = (1.0 - organization).clamp(0.0, 0.85);
    base.mix(&Color::srgb(0.28, 0.3, 0.34), strain)
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
