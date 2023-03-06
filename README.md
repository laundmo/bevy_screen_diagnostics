# bevy_screen_diagnostics

Display bevy diagnostics on the window without any hassle.

<p align="center">
    <img src="https://i.laundmo.com/tENe0/fuxOJOrA74.png/raw">
</p>

What this can do:

- easy frame and entity dignostics
- add custom diagnostics
- change display of diagnostics on the fly
- toggle diagnostics easily

see the [examples](./examples/) on how to do this.

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

## Font

you can use a custom font by disabling the `builtin-font` default feature and providing your own to the `ScreenDiagnosticsPlugin` struct. The builtin font and license can be found in the ./assets/ folder.

## compatible bevy versions

| bevy | bevy_screen_diagnostics |
| ---- | ----------------------- |
| 0.10 | 0.2                     |
| 0.9  | 0.1                     |
