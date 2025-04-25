use bevy::{
    diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use crate::{Aggregate, ScreenDiagnostics};

/// Plugin which adds the bevy [`FrameTimeDiagnosticsPlugin`] and adds its diagnostics to [DiagnosticsText]
///
/// Example: ``16.6 ms/frame 60 fps``
pub struct ScreenFrameDiagnosticsPlugin;

impl Plugin for ScreenFrameDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<FrameTimeDiagnosticsPlugin>() {
            app.add_plugins(FrameTimeDiagnosticsPlugin::default());
        }
        app.add_systems(Startup, setup_frame_diagnostics);
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

/// Plugin which adds the bevy [`EntityCountDiagnosticsPlugin`] and adds its diagnostics to [DiagnosticsText]
///
/// Example: ``15 entities``
pub struct ScreenEntityDiagnosticsPlugin;

impl Plugin for ScreenEntityDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EntityCountDiagnosticsPlugin>() {
            app.add_plugins(EntityCountDiagnosticsPlugin);
        }
        app.add_systems(Startup, setup_entity_diagnostics);
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
#[cfg(feature = "sysinfo_plugin")]
pub(crate) mod sysinfo_plugin {
    use bevy::{diagnostic::SystemInformationDiagnosticsPlugin, prelude::*};

    use crate::{Aggregate, ScreenDiagnostics};
    /// Plugin which adds the bevy [`SystemInformationDiagnosticsPlugin`] and adds its diagnostics to [DiagnosticsText].
    /// "Total" is the value of the entire machine.
    ///
    /// Example: ``09.8% Memory Total 01.1% Memory 12.5% CPU Total 01.5% CPU``
    pub struct ScreenSystemInformationDiagnosticsPlugin;

    impl Plugin for ScreenSystemInformationDiagnosticsPlugin {
        fn build(&self, app: &mut App) {
            if !app.is_plugin_added::<SystemInformationDiagnosticsPlugin>() {
                app.add_plugins(SystemInformationDiagnosticsPlugin);
            }
            app.add_systems(Startup, setup_systeminfo_diagnostics);
        }
    }

    fn setup_systeminfo_diagnostics(mut diags: ResMut<ScreenDiagnostics>) {
        diags
            .add(
                "CPU".to_string(),
                SystemInformationDiagnosticsPlugin::PROCESS_CPU_USAGE,
            )
            .aggregate(Aggregate::Value)
            .format(|v| format!("{v:0>4.1}%"));
        diags
            .add(
                "Memory".to_string(),
                SystemInformationDiagnosticsPlugin::PROCESS_MEM_USAGE,
            )
            .aggregate(Aggregate::Value)
            .format(|v| format!("{v:0>4.1}%"));
        diags
            .add(
                "CPU Total".to_string(),
                SystemInformationDiagnosticsPlugin::SYSTEM_CPU_USAGE,
            )
            .aggregate(Aggregate::Value)
            .format(|v| format!("{v:0>4.1}%"));
        diags
            .add(
                "Memory Total".to_string(),
                SystemInformationDiagnosticsPlugin::SYSTEM_CPU_USAGE,
            )
            .aggregate(Aggregate::Value)
            .format(|v| format!("{v:0>4.1}%"));
    }
}
