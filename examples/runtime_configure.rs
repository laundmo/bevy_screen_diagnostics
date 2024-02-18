// Example of configuring

use bevy::prelude::*;

use bevy_screen_diagnostics::{
    Aggregate, ScreenDiagnostics, ScreenDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin,
};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins)
        .add_plugins(ScreenDiagnosticsPlugin::default())
        .add_plugins(ScreenFrameDiagnosticsPlugin)
        .add_systems(Startup, setup_camera)
        .add_systems(Update, rainbow)
        .add_systems(Update, mouse)
        .run();
}

// need a camera to display the UI
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn rainbow(mut diags: ResMut<ScreenDiagnostics>, mut hue: Local<f32>) {
    diags
        .modify("fps")
        .diagnostic_color(Color::hsl(*hue, 1., 0.5))
        .name_color(Color::hsl(*hue, 0.5, 0.5));
    *hue = (*hue + 1.) % 360.;
}

fn mouse(
    buttons: Res<ButtonInput<MouseButton>>,
    mut diags: ResMut<ScreenDiagnostics>,
    mut aggregate_toggle: Local<bool>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        diags.modify("ms/frame").toggle();
    }
    if buttons.just_pressed(MouseButton::Right) {
        if *aggregate_toggle {
            diags
                .modify("fps")
                .aggregate(Aggregate::Value)
                .format(|v| format!("{v:.0}"));
        } else {
            diags
                .modify("fps")
                .aggregate(Aggregate::Average)
                .format(|v| format!("{v:.3}"));
        }
        *aggregate_toggle = !*aggregate_toggle;
    }
}
