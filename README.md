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
        .add_plugins(ScreenDiagnosticsPlugin::default())
        .add_plugins(ScreenFrameDiagnosticsPlugin)
        .add_startup_system(setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
```

The ScreenFrameDiagnosticsPlugin is a [very simple plugin](./src/extras.rs)

## Plugins

bevy_screen_diagnostics provides the following bevy plugins:
- [`ScreenDiagnostics`]  which offers the basic functionality of displaying diagnostics.
- [`ScreenFrameDiagnosticsPlugin`] display the framerate and frametime (also adds the corresponding bevy diagnostic plugin)
- [`ScreenEntityDiagnosticsPlugin`] display the amount of entities (also adds the corresponding bevy diagnostic plugin)

## Font

This crate uses bevy's default font (a stripped version of FiraCode) through the `builtin-font` default feature.
You can provide your own font while initialising the `ScreenDiagnosticsPlugin` by passing it a asset file path. 

## compatible bevy versions

| bevy | bevy_screen_diagnostics |
| ---- | ----------------------- |
| 0.13 | 0.5                     |
| 0.12 | 0.4                     |
| 0.11 | 0.3                     |
| 0.10 | 0.2                     |
| 0.9  | 0.1                     |
