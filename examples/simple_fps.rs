/// Show frametimes and framerate
use bevy::prelude::*;

use bevy_screen_diagnostics::{ScreenDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ScreenDiagnosticsPlugin::default())
        .add_plugins(ScreenFrameDiagnosticsPlugin)
        .add_systems(Startup, setup_camera)
        .run();
}

// need a camera to display the UI
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
