use bevy::prelude::*;

use bevy_screen_diagnostics::{
    Aggregate, ScreenDiagnostics, ScreenDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ScreenDiagnosticsPlugin::default())
        .add_plugin(ScreenFrameDiagnosticsPlugin)
        .add_startup_system(setup_camera)
        .add_system(rainbow)
        .add_system(mouse)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

fn rainbow(mut diags: ResMut<ScreenDiagnostics>, mut hue: Local<f32>) {
    diags.modify("fps").name_color(Color::hsl(*hue, 1., 0.5));
    *hue = (*hue + 1.) % 360.;
}

fn mouse(
    buttons: Res<Input<MouseButton>>,
    mut diags: ResMut<ScreenDiagnostics>,
    mut aggregate_toggle: Local<bool>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        diags.modify("fps").toggle_name();
    }
    if buttons.just_pressed(MouseButton::Right) {
        if *aggregate_toggle {
            diags
                .modify("fps")
                .aggregate(Aggregate::Value)
                .format(|v| format!("{:.0}", v));
        } else {
            diags
                .modify("fps")
                .aggregate(Aggregate::Average)
                .format(|v| format!("{:.3}", v));
        }
        *aggregate_toggle = !*aggregate_toggle;
    }
}
