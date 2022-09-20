use bevy::{
    diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin},
    prelude::{App, Plugin, ResMut},
};

use crate::{Aggregate, DiagnosticsText};

/// Plugin which adds the [FrameTimeDiagnosticsPlugin] and adds its diagnostics to [DiagnosticsText]
///
/// Example: ``16.6 ms/frame 60 fps``
pub struct ScreenFrameDiagnostics;

impl Plugin for ScreenFrameDiagnostics {
    fn build(&self, app: &mut App) {
        app.add_plugin(FrameTimeDiagnosticsPlugin)
            .add_startup_system(setup_frame_diagnostics);
    }
}

fn setup_frame_diagnostics(mut diags: ResMut<DiagnosticsText>) {
    diags.add(
        "fps".to_string(),
        FrameTimeDiagnosticsPlugin::FPS,
        Aggregate::Value,
        |v| format!("{:.0}", v),
    );

    diags.add(
        "ms/frame".to_string(),
        FrameTimeDiagnosticsPlugin::FRAME_TIME,
        Aggregate::MovingAverage(5),
        |v| format!("{:.2}", v * 1000.),
    );
}

/// Plugin which adds the [EntityCountDiagnosticsPlugin] and adds its diagnostics to [DiagnosticsText]
///
/// Example: ``15 entities``
pub struct ScreenEntityDiagnostics;

impl Plugin for ScreenEntityDiagnostics {
    fn build(&self, app: &mut App) {
        app.add_plugin(EntityCountDiagnosticsPlugin)
            .add_startup_system(setup_entity_diagnostics);
    }
}

fn setup_entity_diagnostics(mut diags: ResMut<DiagnosticsText>) {
    diags.add(
        "entities".to_string(),
        EntityCountDiagnosticsPlugin::ENTITY_COUNT,
        Aggregate::Value,
        |v| format!("{:.0}", v),
    );
}
