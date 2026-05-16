mod lab;

use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};

use lab::settings::LabSettings;
use lab::setup::setup;
use lab::simulation::run_fmp_layer_1;
use lab::ui::lab_ui;
use lab::visuals::update_soldier_visuals;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.035, 0.04, 0.05)))
        .insert_resource(LabSettings::default())
        .add_plugins((DefaultPlugins, EguiPlugin::default()))
        .add_systems(Startup, setup)
        .add_systems(EguiPrimaryContextPass, lab_ui)
        .add_systems(Update, (run_fmp_layer_1, update_soldier_visuals).chain())
        .run();
}
