use bevy::prelude::*;

use super::model::{
    Formation, FormationField, FormationHero, FormationMaterial, FormationSide, SlotVisual,
    SoldierSlot,
};
use super::avian::attach_contact_zone_collider;
use super::scenario::{scenario_origin, scenario_preset};
use super::settings::LabSettings;
use super::visuals::side_color;

const COLUMNS: usize = 9;
const ROWS: usize = 5;

pub fn setup(
    settings: Res<LabSettings>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let scenario = settings.scenario;
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

    let (red_profile, red_material) = scenario_preset(scenario, FormationSide::Red);
    let (blue_profile, blue_material) = scenario_preset(scenario, FormationSide::Blue);

    spawn_formation(
        &mut commands,
        &mut materials,
        &soldier_mesh,
        "Red formation",
        Formation {
            side: FormationSide::Red,
            profile: red_profile,
            origin: scenario_origin(scenario, FormationSide::Red),
            forward: Vec3::X,
            columns: COLUMNS,
            rows: ROWS,
        },
        red_material,
    );

    spawn_formation(
        &mut commands,
        &mut materials,
        &soldier_mesh,
        "Blue formation",
        Formation {
            side: FormationSide::Blue,
            profile: blue_profile,
            origin: scenario_origin(scenario, FormationSide::Blue),
            forward: -Vec3::X,
            columns: COLUMNS,
            rows: ROWS,
        },
        blue_material,
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

    let hero_row = formation.rows / 2;
    let hero_column = formation.front_column();

    for row in 0..formation.rows {
        for column in 0..formation.columns {
            let slot_index = row * formation.columns + column;
            let base_color = side_color(formation.side, 0.0, false);
            let slot_material = materials.add(StandardMaterial {
                base_color,
                perceptual_roughness: 0.65,
                ..default()
            });

            let mut soldier = commands.spawn((
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

            if column == formation.front_column() {
                attach_contact_zone_collider(&mut soldier, formation.side);
            }

            if formation.side == FormationSide::Red && column == hero_column && row == hero_row {
                soldier.insert(FormationHero);
            }
        }
    }
}
