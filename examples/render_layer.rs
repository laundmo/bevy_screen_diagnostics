/// Show frametimes and framerate
use bevy::{prelude::*, render::view::RenderLayers};

use bevy_screen_diagnostics::{ScreenDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ScreenDiagnosticsPlugin {
            render_layer: RenderLayers::from_layers(&[1]), // render diagnostics to different layer
            ..default()
        })
        .add_plugins(ScreenFrameDiagnosticsPlugin)
        .add_systems(Startup, setup_camera)
        .run();
}

// need a camera to display the UI
fn setup_camera(mut commands: Commands) {
    // spawn camera on different layer
    commands.spawn((Camera2d, RenderLayers::from_layers(&[1])));
    // could be useful for rendre-to-texture or editor-like applications
}
