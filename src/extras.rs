use bevy::{
    diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin},
    prelude::{App, Plugin, ResMut},
};

use crate::{Aggregate, ScreenDiagnostics};

/// Plugin which adds the [FrameTimeDiagnosticsPlugin] and adds its diagnostics to [DiagnosticsText]
///
/// Example: ``16.6 ms/frame 60 fps``
pub struct ScreenFrameDiagnosticsPlugin;

impl Plugin for ScreenFrameDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<FrameTimeDiagnosticsPlugin>() {
            app.add_plugin(FrameTimeDiagnosticsPlugin);
        }
        app.add_startup_system(setup_frame_diagnostics);
    }
}

fn setup_frame_diagnostics(mut diags: ResMut<ScreenDiagnostics>) {
    diags
        .add("fps".to_string(), FrameTimeDiagnosticsPlugin::FPS)
        .aggregate(Aggregate::Value)
        .format(|v| format!("{v:.0}"));

    diags
        .add(
            "ms/frame".to_string(),
            FrameTimeDiagnosticsPlugin::FRAME_TIME,
        )
        .aggregate(Aggregate::MovingAverage(5))
        .format(|v| format!("{v:.2}"));
}

/// Plugin which adds the [EntityCountDiagnosticsPlugin] and adds its diagnostics to [DiagnosticsText]
///
/// Example: ``15 entities``
pub struct ScreenEntityDiagnosticsPlugin;

impl Plugin for ScreenEntityDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EntityCountDiagnosticsPlugin>() {
            app.add_plugin(EntityCountDiagnosticsPlugin);
        }
        app.add_startup_system(setup_entity_diagnostics);
    }
}

fn setup_entity_diagnostics(mut diags: ResMut<ScreenDiagnostics>) {
    diags
        .add(
            "entities".to_string(),
            EntityCountDiagnosticsPlugin::ENTITY_COUNT,
        )
        .aggregate(Aggregate::Value)
        .format(|v| format!("{v:.0}"));
}
