/// Add and show all pre-defined simple plugins. Run with feature sysinfo_plugin to get system info like CPU usage.
use bevy::prelude::*;

use bevy_screen_diagnostics::{
    ScreenDiagnosticsPlugin, ScreenEntityDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin,
};

#[cfg(feature = "sysinfo_plugin")]
use bevy_screen_diagnostics::ScreenSystemInformationDiagnosticsPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(ScreenDiagnosticsPlugin::default())
        .add_plugins(ScreenFrameDiagnosticsPlugin)
        .add_plugins(ScreenEntityDiagnosticsPlugin)
        .add_systems(Startup, setup_camera);

    #[cfg(feature = "sysinfo_plugin")]
    app.add_plugins(ScreenSystemInformationDiagnosticsPlugin);

    app.run();
}

// need a camera to display the UI
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
