use bevy::prelude::*;

use bevy_screen_diagnostics::{ScreenDiagnostics, ScreenFrameDiagnostics};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ScreenDiagnostics::default())
        .add_plugin(ScreenFrameDiagnostics)
        .add_startup_system(setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}
