/// Add a custom diagnostic to bevy and to your screen diagnostics
use bevy::{
    diagnostic::{Diagnostic, DiagnosticPath, Diagnostics, RegisterDiagnostic},
    prelude::*,
};

use bevy_screen_diagnostics::{Aggregate, ScreenDiagnostics, ScreenDiagnosticsPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .register_diagnostic(Diagnostic::new(BOX_COUNT))
        .add_plugins(ScreenDiagnosticsPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_diagnostic)
        .add_systems(Update, thing_count)
        .run();
}

#[derive(Component)]
struct Thing;

fn setup(mut commands: Commands) {
    // need a camera to display the UI
    commands.spawn(Camera2dBundle::default());
    // spawn 10 things
    for i in 0..10 {
        commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::WHITE,
                    custom_size: Some(Vec2::new(5.0, 5.0)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(
                    i as f32 * 10.0,
                    0.0,
                    i as f32 * 10.0,
                )),
                ..default()
            })
            .insert(Thing);
    }
}

// For a full explanation on adding custom diagnostics, see: https://github.com/bevyengine/bevy/blob/main/examples/diagnostics/custom_diagnostic.rs
const BOX_COUNT: DiagnosticPath = DiagnosticPath::const_new("box_count");

fn setup_diagnostic(mut onscreen: ResMut<ScreenDiagnostics>) {
    onscreen
        .add("things".to_string(), BOX_COUNT)
        .aggregate(Aggregate::Value)
        .format(|v| format!("{v:.0}"));
}

fn thing_count(mut diagnostics: Diagnostics, parts: Query<&Thing>) {
    diagnostics.add_measurement(&BOX_COUNT, || parts.iter().len() as f64);
}
