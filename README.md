# bevy_screen_diagnostics

Display bevy diagnostics on the window without any hassle.

<p align="center">
    <img src="https://i.laundmo.com/tENe0/fuxOJOrA74.png/raw">
</p>

## Quickstart

This adds the framerate and frametime diagnostics to your window.

```rs
use bevy::prelude::*;

use bevy_screen_diagnostics::{ScreenDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ScreenDiagnosticsPlugin::default())
        .add_plugin(ScreenFrameDiagnosticsPlugin)
        .add_startup_system(setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
```

The ScreenFrameDiagnosticsPlugin is a [very simple plugin](./src/extras.rs)
