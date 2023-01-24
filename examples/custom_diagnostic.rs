/// Add a custom diagnostic to bevy and to your screen diagnostics
use bevy::{
    diagnostic::{Diagnostic, DiagnosticId, Diagnostics},
    prelude::*,
};

use bevy_screen_diagnostics::{Aggregate, ScreenDiagnostics, ScreenDiagnosticsPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ScreenDiagnosticsPlugin::default())
        .add_startup_system(setup)
        .add_startup_system(setup_diagnostic)
        .add_system(thing_count)
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
const UNEVEN_BOX_COUNT: DiagnosticId = DiagnosticId::from_u128(123746129308746521389345767461);

fn setup_diagnostic(mut diagnostics: ResMut<Diagnostics>, mut onscreen: ResMut<ScreenDiagnostics>) {
    diagnostics.add(Diagnostic::new(UNEVEN_BOX_COUNT, "particle_count", 20));
    onscreen
        .add("things".to_string(), UNEVEN_BOX_COUNT)
        .aggregate(Aggregate::Value)
        .format(|v| format!("{v:.0}"));
}

fn thing_count(mut diagnostics: ResMut<Diagnostics>, parts: Query<&Thing>) {
    diagnostics.add_measurement(UNEVEN_BOX_COUNT, || parts.iter().len() as f64);
}
