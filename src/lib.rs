//! A small bevy plugin to easily display diagnostics to the screen
//!
//! FPS and Frametime display:
//! ```rust
//! # use bevy::prelude::*;
//! use bevy_screen_diagnostics::{ScreenDiagnostics, ScreenFrameDiagnostics};
//!
//!
//! fn main() {
//!    App::new()
//!        .add_plugin(DefaultPlugins)
//!        .add_plugin(ScreenDiagnostics)
//!        .add_plugin(ScreenFrameDiagnostics);
//! }
//! ```
//!
//! bevy_screen_diagnostics provides the following bevy plugins:
//! - [ScreenDiagnostics]  which offers the basic functionality of displaying diagnostics.
//! - [ScreenFrameDiagnostics] which adds the [FrameTimeDiagnosticsPlugin] and adds its diagnostics to [DiagnosticsText]
//! - [ScreenEntityDiagnostics] which adds the [EntityCountDiagnosticsPlugin] and adds its diagnostics to [DiagnosticsText]

#![warn(missing_docs)]
#![warn(rustdoc::missing_doc_code_examples)]

use std::collections::BTreeMap;

use bevy::{
    diagnostic::{DiagnosticId, Diagnostics},
    prelude::*,
    time::FixedTimestep,
};

mod extras;

pub use self::extras::{ScreenEntityDiagnostics, ScreenFrameDiagnostics};

const TIMESTEP_10_PER_SECOND: f64 = 1.0 / 10.0;

/// Plugin for displaying Diagnostics on screen.
pub struct ScreenDiagnostics {
    /// The rate at which the diagnostics on screen are updated. Default: 1.0/10.0 (10 times per second).
    pub timestep: f64,
    /// The Style used to position the Text.
    ///
    /// By default this is in the bottom right corner of the window:
    /// ```rust
    /// Style {
    ///     align_self: AlignSelf::FlexEnd,
    ///     position_type: PositionType::Absolute,
    ///     position: UiRect {
    ///         bottom: Val::Px(5.0),
    ///         right: Val::Px(15.0),
    ///         ..default()
    ///     },
    ///     ..default()
    /// }
    /// ```
    pub style: Style,
    /// Colors to use for the description and diagnostic text.
    ///
    /// Will loop back to the start if its shorter than the amount of [DiagnosticId]s added to [DiagnosticsText].
    pub colors: Vec<(Color, Color)>,
    /// The font used for the text. By default [FiraCodeBold](https://github.com/tonsky/FiraCode) is used.
    pub font: Option<&'static str>,
}

const DEFAULT_COLORS: (Color, Color) = (Color::RED, Color::WHITE);

impl Default for ScreenDiagnostics {
    fn default() -> Self {
        Self {
            timestep: TIMESTEP_10_PER_SECOND,
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: UiRect {
                    bottom: Val::Px(5.0),
                    right: Val::Px(15.0),
                    ..default()
                },
                ..default()
            },
            colors: vec![DEFAULT_COLORS],
            font: None,
        }
    }
}

struct FontOption(Option<&'static str>);
struct DiagnosticsStyle(Style);

impl Plugin for ScreenDiagnostics {
    fn build(&self, app: &mut App) {
        app.init_resource::<DiagnosticsText>()
            .insert_resource(FontOption(self.font))
            .init_resource::<ScreenDiagnosticsFont>()
            .insert_resource(DiagnosticsStyle(self.style.clone()))
            .add_startup_system(spawn_ui)
            .add_system(update_onscreen_diags_layout)
            .add_system_set(
                SystemSet::new()
                    // This prints out "goodbye world" twice every second
                    .with_run_criteria(FixedTimestep::step(TIMESTEP_10_PER_SECOND))
                    .with_system(update_diags),
            );
    }
}

// implementation adjusted from: https://github.com/nicopap/bevy-debug-text-overlay/blob/c929111aeff46fbf3a26ceaf714caebd62d87518/src/overlay.rs#L184-L188
struct ScreenDiagnosticsFont(Handle<Font>);
impl FromWorld for ScreenDiagnosticsFont {
    fn from_world(world: &mut World) -> Self {
        let font = world.get_resource::<FontOption>().unwrap();
        let assets = world.get_resource::<AssetServer>().unwrap();
        let font = match font.0 {
            Some(font) => assets.load(font),
            #[cfg(feature = "no-builtin-font")]
            None => panic!(
                "No default font supplied, please either set the `builtin-font` \
                 flag or provide your own font file by setting the `font` field of \
                 `OverlayPlugin` to `Some(thing)`"
            ),
            #[cfg(not(feature = "builtin-font"))]
            None => world.get_resource_mut::<Assets<Font>>().unwrap().add(
                Font::try_from_bytes(include_bytes!("../assets/FiraCodeBold.ttf").to_vec())
                    .expect("The hardcoded builtin font is valid, this should never fail."),
            ),
        };
        Self(font)
    }
}

#[derive(Component)]
struct DiagnosticsTextMarker;

/// Aggregaes which can be used for displaying Diagnostics.
#[derive(Copy, Clone, Default)]
pub enum Aggregate {
    /// The latest [Diagnostic::value]
    #[default]
    Value,
    /// The [Diagnostic::average] of all recorded diagnostic measurements.
    #[allow(dead_code)]
    Average,
    /// A moving average over n last diagnostic measurements.
    ///
    /// If this is larger than the amount of diagnostic measurement stored for that diagnostic, no update will happen.
    MovingAverage(usize),
}

/// Type alias for the fuction used to format a diagnostic value to a string.
///
/// Useful especially for applying some operations to the value before formatting.
///
/// Example: ``|v| format!("{:.2}", v);`` which limits the decimal places to 1.
pub type FormatFn = fn(f64) -> String;

/// Resource which maps the name to the [DiagnosticId], [Aggregate] and [ConvertFn]
#[derive(Default)]
pub struct DiagnosticsText {
    diagnostics: BTreeMap<String, (DiagnosticId, Aggregate, FormatFn)>,
    layout_changed: bool,
    colors: Vec<(Color, Color)>,
    color_index: usize,
}

impl DiagnosticsText {
    /// Add a diagnostic to be displayed.
    ///
    /// * `name` - The name displayed on-screen. Also used as a key.
    /// * `diagnostic` - The [DiagnosticId] which is displayed.
    /// * `aggregate` - The Aggregate which is applied to the diagnostic measurements.
    /// * `format` - A function with the signature fn(f64) -> String used to transform the diagnostic result into a string.
    ///              Convert
    /// ```rust
    /// diags.add(
    ///     "ms/frame".to_string(),
    ///     FrameTimeDiagnosticsPlugin::FRAME_TIME,
    ///     Aggregate::MovingAverage(5),
    ///     Some(|n| n * 1000.),
    /// );
    /// ```
    pub fn add(
        &mut self,
        name: String,
        diagnostic: DiagnosticId,
        aggregate: Aggregate,
        format_fn: FormatFn,
    ) {
        self.diagnostics
            .insert(name, (diagnostic, aggregate, format_fn));
        self.layout_changed = true;
    }

    /// Remove a diagnostic from the screen by name.
    #[allow(dead_code)]
    pub fn remove(&mut self, name: String) {
        self.diagnostics.remove(&name);
        self.layout_changed = true;
    }

    fn update(&self, diagnostics: Res<Diagnostics>, mut text: Mut<Text>) {
        if self.layout_changed {
            return;
        }

        for (i, (diag_id, aggregate, transform_fn)) in self.diagnostics.values().rev().enumerate() {
            if let Some(diag) = diagnostics.get(*diag_id) {
                let diag_val = match aggregate {
                    Aggregate::Value => diag.value(),
                    Aggregate::Average => diag.average(),
                    Aggregate::MovingAverage(count) => {
                        let skip_maybe = diag.history_len().checked_sub(*count);
                        skip_maybe.map(|skip| diag.values().skip(skip).sum::<f64>() / *count as f64)
                    }
                };
                if let Some(val) = diag_val {
                    text.sections[i * 2].value = transform_fn(val);
                }
            }
        }
    }

    fn rebuild(&mut self, font: Res<ScreenDiagnosticsFont>) -> Text {
        let mut sections: Vec<TextSection> = Vec::with_capacity(self.diagnostics.len());
        if self.colors.is_empty() {
            warn!(
                "ScreenDiagnostics.colors is empty, assuming default colors - please provide at least one pair of colors."
            );
            self.colors.push(DEFAULT_COLORS);
        }
        for name in self.diagnostics.keys().rev() {
            let colors = self.colors[self.color_index];
            sections.append(&mut self.section(name, font.0.clone(), colors));
            self.color_index = (self.color_index + 1) % self.colors.len(); // loop around
        }

        Text {
            sections,
            ..default()
        }
    }

    fn section(
        &self,
        name: &str,
        font: Handle<Font>,
        (diag, text): (Color, Color),
    ) -> Vec<TextSection> {
        vec![
            TextSection {
                value: "".to_string(),
                style: TextStyle {
                    font: font.clone(),
                    font_size: 20.0,
                    color: diag,
                },
            },
            TextSection {
                value: format!(" {} ", name),
                style: TextStyle {
                    font,
                    font_size: 20.0,
                    color: text,
                },
            },
        ]
    }
}

fn spawn_ui(mut commands: Commands, diag_style: Res<DiagnosticsStyle>) {
    commands
        .spawn_bundle(TextBundle {
            style: diag_style.0.clone(),
            text: Text {
                sections: vec![],
                ..default()
            },
            ..default()
        })
        .insert(DiagnosticsTextMarker);
}

fn update_onscreen_diags_layout(
    mut diags: ResMut<DiagnosticsText>,
    font: Res<ScreenDiagnosticsFont>,
    mut query: Query<&mut Text, With<DiagnosticsTextMarker>>,
) {
    if diags.layout_changed {
        let mut text = query.single_mut();
        *text = diags.rebuild(font);
        diags.layout_changed = false;
    }
}

fn update_diags(
    diag: ResMut<DiagnosticsText>,
    diagnostics: Res<Diagnostics>,
    mut query: Query<&mut Text, With<DiagnosticsTextMarker>>,
) {
    let text = query.single_mut();
    diag.update(diagnostics, text);
}
