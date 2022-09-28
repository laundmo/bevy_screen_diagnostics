use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};

use bevy_screen_diagnostics::{Aggregate, ScreenDiagnostics, ScreenDiagnosticsPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ScreenDiagnosticsPlugin {
            style: Style {
                margin: UiRect::all(Val::Auto),
                align_self: AlignSelf::Center,
                ..default()
            },
            ..default()
        })
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_startup_system(setup_camera)
        .add_startup_system(setup_frame_diagnostics)
        .run();
}

fn setup_frame_diagnostics(mut diags: ResMut<ScreenDiagnostics>) {
    diags
        .add("fps".to_string(), FrameTimeDiagnosticsPlugin::FPS)
        .aggregate(Aggregate::Value)
        .format(|v| format!("{:.0}", v));
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}
