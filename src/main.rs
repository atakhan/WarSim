mod lab;

use avian3d::schedule::{PhysicsSchedule, PhysicsStepSystems};
use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};

use lab::avian::{
    AvianContactCache, collect_avian_contacts, configure_avian_physics,
    sync_contact_collider_transforms,
};
use lab::motion::update_formation_motion;
use lab::settings::LabSettings;
use lab::setup::setup;
use lab::simulation::{GicLabState, run_fmp_layer_1};
use lab::ui::lab_ui;
use lab::visuals::update_soldier_visuals;

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.035, 0.04, 0.05)))
        .insert_resource(LabSettings::default())
        .init_resource::<AvianContactCache>()
        .init_resource::<GicLabState>()
        .add_plugins((DefaultPlugins, EguiPlugin::default()));

    configure_avian_physics(&mut app);

    app.add_systems(Startup, setup)
        .add_systems(EguiPrimaryContextPass, lab_ui)
        .add_systems(
            Update,
            (
                update_formation_motion,
                sync_contact_collider_transforms,
            )
                .chain(),
        )
        .add_systems(
            PhysicsSchedule,
            collect_avian_contacts.after(PhysicsStepSystems::Last),
        )
        .add_systems(
            PostUpdate,
            (run_fmp_layer_1, update_soldier_visuals).chain(),
        )
        .run();
}
