use bevy::prelude::*;

use super::model::{
    Formation, FormationField, FormationMaterial, FormationSide, PressureProfile, SlotVisual,
    SoldierSlot,
};
use super::scenario::{LabScenario, scenario_origin};
use super::visuals::side_color;

const COLUMNS: usize = 9;
const ROWS: usize = 5;

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 13.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 8_000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-3.0, 8.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let ground_mesh = meshes.add(Cuboid::new(16.0, 0.05, 10.0));
    let ground_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.08, 0.095, 0.09),
        perceptual_roughness: 0.9,
        ..default()
    });

    commands.spawn((
        Mesh3d(ground_mesh),
        MeshMaterial3d(ground_material),
        Transform::from_xyz(0.0, -0.05, 0.0),
    ));

    let soldier_mesh = meshes.add(Cuboid::new(0.42, 0.24, 0.42));

    spawn_formation(
        &mut commands,
        &mut materials,
        &soldier_mesh,
        "Red wedge",
        Formation {
            side: FormationSide::Red,
            profile: PressureProfile::Wedge,
            origin: scenario_origin(LabScenario::WedgeVsLine, FormationSide::Red),
            forward: Vec3::X,
            columns: COLUMNS,
            rows: ROWS,
        },
        FormationMaterial {
            stiffness: 7.0,
            forward_multiplier: 1.5,
            lateral_multiplier: 0.5,
            yield_strength: 5.3,
            viscosity: 0.9,
            morale: 0.82,
            fatigue: 0.12,
        },
    );

    spawn_formation(
        &mut commands,
        &mut materials,
        &soldier_mesh,
        "Blue line",
        Formation {
            side: FormationSide::Blue,
            profile: PressureProfile::Line,
            origin: scenario_origin(LabScenario::WedgeVsLine, FormationSide::Blue),
            forward: -Vec3::X,
            columns: COLUMNS,
            rows: ROWS,
        },
        FormationMaterial {
            stiffness: 6.2,
            forward_multiplier: 1.0,
            lateral_multiplier: 1.0,
            yield_strength: 5.8,
            viscosity: 1.1,
            morale: 0.9,
            fatigue: 0.08,
        },
    );
}

fn spawn_formation(
    commands: &mut Commands,
    materials: &mut Assets<StandardMaterial>,
    soldier_mesh: &Handle<Mesh>,
    name: &'static str,
    formation: Formation,
    material: FormationMaterial,
) {
    let field = FormationField::new(formation.columns, formation.rows);
    let formation_entity = commands
        .spawn((Name::new(name), formation, material, field))
        .id();

    for row in 0..formation.rows {
        for column in 0..formation.columns {
            let slot_index = row * formation.columns + column;
            let base_color = side_color(formation.side, 0.0, false);
            let slot_material = materials.add(StandardMaterial {
                base_color,
                perceptual_roughness: 0.65,
                ..default()
            });

            commands.spawn((
                Mesh3d(soldier_mesh.clone()),
                MeshMaterial3d(slot_material.clone()),
                Transform::default(),
                SoldierSlot {
                    formation: formation_entity,
                    column,
                    row,
                    index: slot_index,
                },
                SlotVisual {
                    material: slot_material,
                },
            ));
        }
    }
}
