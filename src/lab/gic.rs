use avian3d::prelude::*;
use bevy::prelude::*;

use super::avian::{ContactZoneCollider, spatial_query_blue_contacts};
use super::contact::{ContactBoundary, ContactRowRange, ContactSample};
use super::model::{Formation, FormationSide, SoldierSlot};

/// Параметры одного thrust-удара (GIC).
#[derive(Clone, Copy, Debug)]
pub struct GicThrustParams {
    pub origin: Vec3,
    pub direction: Vec3,
    pub length: f32,
    pub weapon_mass: f32,
    pub tip_velocity: f32,
}

impl GicThrustParams {
    pub fn from_hero(hero_position: Vec3, attacker: &Formation) -> Self {
        Self {
            origin: hero_position,
            direction: attacker.forward.normalize_or_zero(),
            length: 2.8,
            weapon_mass: 1.2,
            tip_velocity: 4.0,
        }
    }
}

/// Точка на переднем ряду защитника (harness / unit tests).
#[cfg(test)]
#[derive(Clone, Copy, Debug)]
pub struct DefenderFrontSample {
    pub row: usize,
    pub position: Vec3,
}

/// Импульс удара → граничное давление.
#[derive(Clone, Copy, Debug)]
pub struct GicImpulse {
    pub target_side: FormationSide,
    pub front_column: usize,
    pub rows: usize,
    pub row_range: ContactRowRange,
    pub pressure_boost: f32,
    pub compression: f32,
    pub disruption: f32,
    pub from_shapecast: bool,
}

#[cfg(test)]
/// Радиус поперечного попадания (геометрический fallback).
pub const THRUST_LATERAL_RADIUS: f32 = 0.38;

pub const IMPULSE_TO_PRESSURE: f32 = 0.22;

/// Полуразмеры swept cuboid для Avian shapecast (ось X локальная → выравнивается по direction).
pub const THRUST_CAST_HALF_X: f32 = 0.14;
pub const THRUST_CAST_HALF_Y: f32 = 0.11;
pub const THRUST_CAST_HALF_Z: f32 = 0.2;

#[cfg(test)]
pub fn defender_front_samples(formation: Formation) -> Vec<DefenderFrontSample> {
    let column = formation.front_column();
    (0..formation.rows)
        .map(|row| DefenderFrontSample {
            row,
            position: formation.slot_position(column, row),
        })
        .collect()
}

impl GicImpulse {
    pub fn merge_into_boundary(self, boundary: &mut ContactBoundary) {
        for sample in &mut boundary.samples {
            if sample.column == self.front_column
                && sample.row >= self.row_range.start
                && sample.row < self.row_range.end
            {
                sample.normal_pressure += self.pressure_boost;
            }
        }
    }

    pub fn to_boundary(self) -> ContactBoundary {
        let samples = (self.row_range.start..self.row_range.end.min(self.rows))
            .map(|row| ContactSample {
                column: self.front_column,
                row,
                normal_pressure: self.pressure_boost,
                compression: self.compression,
                disruption: self.disruption,
            })
            .collect();
        ContactBoundary { samples }
    }

    pub fn is_empty(self) -> bool {
        self.row_range.is_empty() || self.pressure_boost <= 0.0
    }
}

/// Runtime: Avian shapecast по синим contact colliders.
pub fn hero_thrust_impulse_shapecast(
    spatial_query: &SpatialQuery,
    hero_entity: Entity,
    thrust: GicThrustParams,
    defender: &Formation,
    slots: &Query<(Entity, &SoldierSlot, Option<&ContactZoneCollider>)>,
) -> Option<GicImpulse> {
    let direction = thrust.direction.normalize_or_zero();
    if direction == Vec3::ZERO || thrust.length <= 0.0 {
        return None;
    }

    let cast_dir = Dir3::new(direction).ok()?;
    let shape = Collider::cuboid(
        THRUST_CAST_HALF_X,
        THRUST_CAST_HALF_Y,
        THRUST_CAST_HALF_Z,
    );
    let rotation = Quat::from_rotation_arc(Vec3::X, direction);
    let cast_origin = thrust.origin - direction * 0.08;
    let config = ShapeCastConfig::from_max_distance(thrust.length);
    let filter = spatial_query_blue_contacts().with_excluded_entities([hero_entity]);

    let hits = spatial_query.shape_hits(
        &shape,
        cast_origin,
        rotation,
        cast_dir,
        24,
        &config,
        &filter,
    );

    let mut row_min = None;
    let mut row_max = 0;
    let mut total_impulse = 0.0_f32;

    for hit in hits {
        let Ok((_, slot, collider)) = slots.get(hit.entity) else {
            continue;
        };
        let Some(collider) = collider else {
            continue;
        };
        if collider.side != FormationSide::Blue {
            continue;
        }

        let depth_factor = (hit.distance / thrust.length).clamp(0.12, 1.0);
        let defender_normal = hit.normal1;
        let angle_factor = direction.dot(-defender_normal).clamp(0.35, 1.0);
        let row_impulse =
            thrust.weapon_mass * thrust.tip_velocity * depth_factor * angle_factor;
        total_impulse += row_impulse;
        row_min = Some(row_min.map_or(slot.row, |m: usize| m.min(slot.row)));
        row_max = row_max.max(slot.row + 1);
    }

    build_impulse_from_rows(
        row_min,
        row_max,
        total_impulse,
        defender,
        true,
    )
}

#[cfg(test)]
pub fn compute_thrust_impulse_geometric(
    thrust: GicThrustParams,
    defender_samples: &[DefenderFrontSample],
    defender: &Formation,
) -> Option<GicImpulse> {
    let direction = thrust.direction.normalize_or_zero();
    if direction == Vec3::ZERO || thrust.length <= 0.0 {
        return None;
    }

    let defender_normal = -direction;
    let mut row_min = None;
    let mut row_max = 0;
    let mut total_impulse = 0.0_f32;

    for sample in defender_samples {
        let rel = sample.position - thrust.origin;
        let along = rel.dot(direction);
        if along < 0.0 || along > thrust.length {
            continue;
        }
        let lateral = (rel - direction * along).length();
        if lateral > THRUST_LATERAL_RADIUS {
            continue;
        }

        let depth_factor = (along / thrust.length).clamp(0.12, 1.0);
        let angle_factor = defender_normal.dot(direction).clamp(0.35, 1.0);
        let row_impulse =
            thrust.weapon_mass * thrust.tip_velocity * depth_factor * angle_factor;
        total_impulse += row_impulse;
        row_min = Some(row_min.map_or(sample.row, |m: usize| m.min(sample.row)));
        row_max = row_max.max(sample.row + 1);
    }

    build_impulse_from_rows(row_min, row_max, total_impulse, defender, false)
}

fn build_impulse_from_rows(
    row_min: Option<usize>,
    row_max: usize,
    total_impulse: f32,
    defender: &Formation,
    from_shapecast: bool,
) -> Option<GicImpulse> {
    let start = row_min?;
    let row_range = ContactRowRange { start, end: row_max };
    if row_range.is_empty() || total_impulse <= 0.0 {
        return None;
    }

    Some(GicImpulse {
        target_side: FormationSide::Blue,
        front_column: defender.front_column(),
        rows: defender.rows,
        row_range,
        pressure_boost: total_impulse * IMPULSE_TO_PRESSURE,
        compression: 0.5,
        disruption: 0.0,
        from_shapecast,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_formation(side: FormationSide, origin: Vec3, forward: Vec3) -> Formation {
        Formation {
            side,
            profile: super::super::field::PressureProfile::Line,
            origin,
            forward,
            columns: 9,
            rows: 5,
        }
    }

    #[test]
    fn geometric_thrust_hits_defender_front_in_range() {
        let red = line_formation(FormationSide::Red, Vec3::new(-2.9, 0.12, 0.0), Vec3::X);
        let blue = line_formation(FormationSide::Blue, Vec3::new(2.9, 0.12, 0.0), Vec3::NEG_X);
        let hero = red.slot_position(red.front_column(), 2);
        let thrust = GicThrustParams::from_hero(hero, &red);
        let impulse =
            compute_thrust_impulse_geometric(thrust, &defender_front_samples(blue), &blue)
                .expect("hit");

        assert!(!impulse.from_shapecast);
        assert!(!impulse.row_range.is_empty());
        assert!(impulse.pressure_boost > 0.0);
    }

    #[test]
    fn gic_boost_raises_field_pressure_over_boundary_alone() {
        use super::super::contact::{ContactRequest, ContactRowRange};
        use super::super::field::{FormationField, PressureProfile};

        let base_request = ContactRequest {
            front_column: 0,
            rows: 5,
            row_range: ContactRowRange::full(5),
            incoming_profile: PressureProfile::Line,
            normal_pressure: 6.0,
            compression: 0.6,
            disruption: 0.0,
        };
        let mut field_base = FormationField::new(3, 5);
        base_request.resolve().apply_to_field(&mut field_base, 0.5);
        let center_base = field_base.pressure[field_base.index(0, 2)];

        let mut field_gic = FormationField::new(3, 5);
        let mut boundary = base_request.resolve();
        let gic = GicImpulse {
            target_side: FormationSide::Blue,
            front_column: 0,
            rows: 5,
            row_range: ContactRowRange { start: 1, end: 4 },
            pressure_boost: 5.0,
            compression: 0.6,
            disruption: 0.0,
            from_shapecast: false,
        };
        gic.merge_into_boundary(&mut boundary);
        boundary.apply_to_field(&mut field_gic, 0.5);

        assert!(field_gic.pressure[field_gic.index(0, 2)] > center_base);
    }

    #[test]
    fn geometric_thrust_misses_when_defender_too_far() {
        let red = line_formation(FormationSide::Red, Vec3::new(-4.0, 0.12, 0.0), Vec3::X);
        let blue = line_formation(FormationSide::Blue, Vec3::new(4.0, 0.12, 0.0), Vec3::NEG_X);
        let hero = red.slot_position(red.front_column(), 2);
        let thrust = GicThrustParams::from_hero(hero, &red);

        assert!(
            compute_thrust_impulse_geometric(thrust, &defender_front_samples(blue), &blue).is_none()
        );
    }
}
