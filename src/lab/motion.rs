use bevy::prelude::*;

use super::model::{Formation, FormationSide};
use super::settings::LabSettings;

pub fn update_formation_motion(
    time: Res<Time>,
    settings: Res<LabSettings>,
    mut formations: Query<&mut Formation>,
) {
    let motion = settings.formation_motion;
    if settings.paused || !motion.is_approaching() {
        return;
    }

    let dt = time.delta_secs().min(1.0 / 20.0);
    let Some(fronts) = collect_front_positions(&formations) else {
        return;
    };

    if !should_advance_approach(
        fronts,
        settings.approach_speed,
        dt,
        motion.advancing_side_count(),
    ) {
        return;
    }

    for mut formation in &mut formations {
        if !motion.advances(formation.side) {
            continue;
        }
        let step = formation.forward * settings.approach_speed * dt;
        formation.origin += step;
    }
}

#[derive(Clone, Copy)]
pub struct FrontPositions {
    pub red: f32,
    pub blue: f32,
}

fn collect_front_positions(formations: &Query<&mut Formation>) -> Option<FrontPositions> {
    let mut red = None;
    let mut blue = None;

    for formation in formations.iter() {
        let front_position = formation.contact_front().front_position;
        match formation.side {
            FormationSide::Red => red = Some(front_position),
            FormationSide::Blue => blue = Some(front_position),
        }
    }

    Some(FrontPositions {
        red: red?,
        blue: blue?,
    })
}

pub fn should_advance_approach(
    fronts: FrontPositions,
    speed: f32,
    dt: f32,
    advancing_sides: u32,
) -> bool {
    if speed <= 0.0 || dt <= 0.0 || advancing_sides == 0 {
        return false;
    }

    let gap = front_gap(fronts);
    gap > advancing_sides as f32 * speed * dt
}

pub fn front_gap(fronts: FrontPositions) -> f32 {
    (fronts.blue - fronts.red).max(0.0)
}

pub fn compression_from_gap(gap: f32, contact_distance: f32) -> f32 {
    (contact_distance - gap).max(0.0)
}

/// Полуширина cuboid-коллайдера переднего ряда (см. `avian::attach_contact_zone_collider`).
pub const CONTACT_COLLIDER_HALF_WIDTH: f32 = 0.21;

/// Сжатие из глубины проникновения collider (Avian penetration или геометрия).
pub fn compression_from_penetration(penetration: f32, contact_distance: f32) -> f32 {
    if penetration <= 0.0 {
        return 0.0;
    }
    let full_scale = CONTACT_COLLIDER_HALF_WIDTH * 2.0;
    (penetration / full_scale * contact_distance).min(contact_distance)
}

pub fn merge_contact_compression(gap: f32, penetration: f32, contact_distance: f32) -> f32 {
    compression_from_gap(gap, contact_distance)
        .max(compression_from_penetration(penetration, contact_distance))
}

/// Масштаб давления от относительной скорости сближения (0 = нет, ~1.35 при сильном ударе).
pub fn impact_scale_from_approach_speed(approach_speed: f32, reference_speed: f32) -> f32 {
    1.0 + (approach_speed / reference_speed.max(0.1)).clamp(0.0, 0.35)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::settings::FormationMotion;

    #[test]
    fn front_gap_is_distance_between_opposing_fronts() {
        let gap = front_gap(FrontPositions {
            red: 1.0,
            blue: 4.0,
        });

        assert!((gap - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn compression_grows_as_gap_shrinks() {
        assert!(compression_from_gap(2.0, 5.0) > compression_from_gap(4.0, 5.0));
    }

    #[test]
    fn penetration_compression_grows_with_depth() {
        assert!(
            compression_from_penetration(0.3, 5.0) > compression_from_penetration(0.1, 5.0)
        );
    }

    #[test]
    fn merge_compression_uses_max_of_gap_and_penetration() {
        let merged = merge_contact_compression(4.0, 0.35, 5.0);
        assert!(merged >= compression_from_gap(4.0, 5.0));
        assert!(merged >= compression_from_penetration(0.35, 5.0));
    }

    #[test]
    fn approach_stops_when_gap_would_cross_zero() {
        let fronts = FrontPositions {
            red: 1.0,
            blue: 1.4,
        };

        assert!(!should_advance_approach(fronts, 1.0, 0.5, 1));
        assert!(!should_advance_approach(fronts, 1.0, 0.5, 2));
        assert!(should_advance_approach(fronts, 0.2, 0.5, 1));
        assert!(should_advance_approach(
            FrontPositions {
                red: 1.0,
                blue: 2.5,
            },
            0.2,
            0.5,
            2,
        ));
    }

    #[test]
    fn one_sided_advance_only_moves_designated_side() {
        assert!(FormationMotion::ApproachRed.advances(FormationSide::Red));
        assert!(!FormationMotion::ApproachRed.advances(FormationSide::Blue));
        assert!(FormationMotion::ApproachBlue.advances(FormationSide::Blue));
        assert!(!FormationMotion::ApproachBlue.advances(FormationSide::Red));
    }
}
