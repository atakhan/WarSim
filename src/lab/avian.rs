use avian3d::prelude::*;
use bevy::prelude::*;

use super::contact::{ContactRequest, ContactRowRange};
use super::model::{Formation, FormationSide, SLOT_SPACING, SoldierSlot};
use super::motion::{
    CONTACT_COLLIDER_HALF_WIDTH, compression_from_gap, compression_from_penetration,
    impact_scale_from_approach_speed, merge_contact_compression,
};
use super::probe::ContactProbeInput;
use super::settings::LabSettings;

/// Маркер: слот переднего ряда участвует в Avian contact zone.
#[derive(Component)]
pub struct ContactZoneCollider {
    pub side: FormationSide,
}

#[derive(Resource, Default)]
pub struct AvianContactCache {
    pub red_defense: Option<AvianContactHit>,
    pub blue_defense: Option<AvianContactHit>,
}

#[derive(Clone, Copy, Debug)]
pub struct AvianContactHit {
    pub row_range: ContactRowRange,
    /// Сжатие от зазора фронтов (synthetic-совместимое).
    pub gap_compression: f32,
    /// Максимальная глубина проникновения collider по оси сближения.
    pub penetration_depth: f32,
    /// Сжатие, выведенное из penetration.
    pub penetration_compression: f32,
    /// Итог для ContactRequest: max(gap, penetration).
    pub compression: f32,
    pub disruption: f32,
    /// Множитель normal_pressure от скорости сближения / импульса.
    pub impact_scale: f32,
}

impl AvianContactCache {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn resolve_request(
        &self,
        input: ContactProbeInput,
        target_side: FormationSide,
    ) -> Option<ContactRequest> {
        let hit = match target_side {
            FormationSide::Red => self.red_defense?,
            FormationSide::Blue => self.blue_defense?,
        };

        if hit.compression <= 0.0 || hit.row_range.is_empty() {
            return None;
        }

        let scale = hit.compression / input.detection.contact_distance.max(f32::EPSILON);

        Some(ContactRequest {
            front_column: input.target.front_column,
            rows: input.target.rows,
            row_range: hit.row_range,
            incoming_profile: input.incoming_profile,
            normal_pressure: input.detection.base_pressure * scale * hit.impact_scale,
            compression: hit.compression,
            disruption: hit.disruption,
        })
    }
}

pub const RED_CONTACT_LAYER: LayerMask = LayerMask(1 << 1);
pub const BLUE_CONTACT_LAYER: LayerMask = LayerMask(1 << 2);

pub fn configure_avian_physics(app: &mut App) {
    app.add_plugins(PhysicsPlugins::default().with_length_unit(1.0));
}

pub fn spatial_query_blue_contacts() -> SpatialQueryFilter {
    SpatialQueryFilter::from_mask(BLUE_CONTACT_LAYER)
}

pub fn attach_contact_zone_collider(commands: &mut EntityCommands, side: FormationSide) {
    let membership = match side {
        FormationSide::Red => CollisionLayers::new(RED_CONTACT_LAYER, BLUE_CONTACT_LAYER),
        FormationSide::Blue => CollisionLayers::new(BLUE_CONTACT_LAYER, RED_CONTACT_LAYER),
    };

    commands.insert((
        ContactZoneCollider { side },
        RigidBody::Kinematic,
        Collider::cuboid(0.21, 0.12, 0.21),
        ColliderDensity(2.0),
        membership,
        CollidingEntities::default(),
    ));
}

pub fn sync_contact_collider_transforms(
    formations: Query<&Formation>,
    mut soldiers: Query<(&SoldierSlot, &ContactZoneCollider, &mut Transform)>,
) {
    for (slot, collider, mut transform) in &mut soldiers {
        let Ok(formation) = formations.get(slot.formation) else {
            continue;
        };
        if slot.column != formation.front_column() {
            continue;
        }

        let row_center = (formation.rows as f32 - 1.0) * 0.5;
        let column_center = (formation.columns as f32 - 1.0) * 0.5;
        let row_offset = (slot.row as f32 - row_center) * SLOT_SPACING;
        let column_offset = (slot.column as f32 - column_center) * SLOT_SPACING;

        transform.translation = formation.origin + Vec3::new(column_offset, 0.12, row_offset);
        transform.rotation = Quat::IDENTITY;
        let _ = collider;
    }
}

pub fn collect_avian_contacts(
    settings: Res<LabSettings>,
    time: Res<Time>,
    mut cache: ResMut<AvianContactCache>,
    formations: Query<&Formation>,
    collisions: Collisions,
    slots: Query<(
        Entity,
        &SoldierSlot,
        &ContactZoneCollider,
        &CollidingEntities,
        &GlobalTransform,
    )>,
) {
    cache.clear();

    let (Some(red_formation), Some(blue_formation)) = (
        formations.iter().find(|f| f.side == FormationSide::Red),
        formations.iter().find(|f| f.side == FormationSide::Blue),
    ) else {
        return;
    };

    let gap = (blue_formation.contact_front().front_position
        - red_formation.contact_front().front_position)
        .abs();
    let gap_compression = compression_from_gap(gap, settings.contact_distance);
    let dt = time.delta_secs().max(f32::EPSILON);

    let motion_speed = if settings.formation_motion.is_approaching() {
        settings.approach_speed
    } else {
        0.0
    };

    let mut red_rows = RowTouchSet::default();
    let mut blue_rows = RowTouchSet::default();
    let mut max_penetration = 0.0_f32;
    let mut max_impact_speed = motion_speed;

    for (entity, slot, collider, colliding, transform) in &slots {
        let self_x = transform.translation().x;
        for other in colliding.iter() {
            let Ok((_, other_slot, other_collider, _, other_transform)) = slots.get(*other) else {
                continue;
            };
            if other_collider.side == collider.side {
                continue;
            }

            let other_x = other_transform.translation().x;
            let (red_x, blue_x) = match collider.side {
                FormationSide::Red => (self_x, other_x),
                FormationSide::Blue => (other_x, self_x),
            };
            let (penetration, impact_speed) =
                sample_engagement_contact(&collisions, entity, *other, red_x, blue_x, dt);
            max_penetration = max_penetration.max(penetration);
            max_impact_speed = max_impact_speed.max(impact_speed);

            match collider.side {
                FormationSide::Blue => blue_rows.touch(slot.row),
                FormationSide::Red => red_rows.touch(slot.row),
            }
            match other_collider.side {
                FormationSide::Blue => blue_rows.touch(other_slot.row),
                FormationSide::Red => red_rows.touch(other_slot.row),
            }
        }
    }

    let penetration_compression =
        compression_from_penetration(max_penetration, settings.contact_distance);
    let compression = merge_contact_compression(gap, max_penetration, settings.contact_distance);
    let impact_scale = impact_scale_from_approach_speed(max_impact_speed, 1.0);

    let metrics = ContactMetrics {
        gap_compression,
        penetration_depth: max_penetration,
        penetration_compression,
        compression,
        impact_scale,
    };

    cache.blue_defense = blue_rows.into_hit(blue_formation.rows, metrics);
    cache.red_defense = red_rows.into_hit(red_formation.rows, metrics);
}

#[derive(Clone, Copy)]
struct ContactMetrics {
    gap_compression: f32,
    penetration_depth: f32,
    penetration_compression: f32,
    compression: f32,
    impact_scale: f32,
}

fn sample_engagement_contact(
    collisions: &Collisions,
    entity: Entity,
    other: Entity,
    red_x: f32,
    blue_x: f32,
    dt: f32,
) -> (f32, f32) {
    let mut penetration = geometric_penetration(red_x, blue_x);
    let mut impact_speed = 0.0_f32;

    if let Some(pair) = collisions.get(entity, other) {
        if let Some(point) = pair.find_deepest_contact() {
            penetration = penetration.max(point.penetration);
            impact_speed = impact_speed.max((-point.normal_speed).max(0.0));
        }
        let impulse = pair.total_normal_impulse_magnitude();
        if impulse > 0.0 && dt > 0.0 {
            impact_speed = impact_speed.max(impulse / dt);
        }
    }

    (penetration, impact_speed)
}

/// Проникновение вдоль оси X (красный справа, синий слева).
pub fn geometric_penetration(red_x: f32, blue_x: f32) -> f32 {
    let half = CONTACT_COLLIDER_HALF_WIDTH;
    ((red_x + half) - (blue_x - half)).max(0.0)
}

#[derive(Default)]
struct RowTouchSet {
    min: Option<usize>,
    max: Option<usize>,
}

impl RowTouchSet {
    fn touch(&mut self, row: usize) {
        self.min = Some(self.min.map_or(row, |m| m.min(row)));
        self.max = Some(self.max.map_or(row + 1, |m| m.max(row + 1)));
    }

    fn into_hit(self, rows: usize, metrics: ContactMetrics) -> Option<AvianContactHit> {
        let start = self.min?;
        let end = self.max?;
        if start >= end || metrics.compression <= 0.0 {
            return None;
        }

        let row_range = ContactRowRange { start, end };
        let overlap_ratio = row_range.len() as f32 / rows.max(1) as f32;

        Some(AvianContactHit {
            row_range,
            gap_compression: metrics.gap_compression,
            penetration_depth: metrics.penetration_depth,
            penetration_compression: metrics.penetration_compression,
            compression: metrics.compression,
            disruption: 1.0 - overlap_ratio,
            impact_scale: metrics.impact_scale,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometric_penetration_positive_when_red_right_of_blue() {
        assert!(geometric_penetration(0.5, 0.2) > 0.0);
    }

    #[test]
    fn geometric_penetration_zero_when_separated() {
        assert!(geometric_penetration(-1.0, 1.0) <= 0.0);
    }
}
