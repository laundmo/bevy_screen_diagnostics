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

pub use self::extras::{ScreenEntityDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin};

const TIMESTEP_10_PER_SECOND: f64 = 1.0 / 10.0;

/// Plugin for displaying Diagnostics on screen.
pub struct ScreenDiagnosticsPlugin {
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
    /// The font used for the text. By default [FiraCodeBold](https://github.com/tonsky/FiraCode) is used.
    pub font: Option<&'static str>,
}

const DEFAULT_COLORS: (Color, Color) = (Color::RED, Color::WHITE);

impl Default for ScreenDiagnosticsPlugin {
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
            font: None,
        }
    }
}

struct FontOption(Option<&'static str>);
struct DiagnosticsStyle(Style);

impl Plugin for ScreenDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScreenDiagnostics>()
            .insert_resource(FontOption(self.font))
            .init_resource::<ScreenDiagnosticsFont>()
            .insert_resource(DiagnosticsStyle(self.style.clone()))
            .add_startup_system(spawn_ui)
            .add_system(update_onscreen_diags_layout)
            .add_system_set(
                SystemSet::new()
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
#[derive(Copy, Clone, Default, Debug)]
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
pub struct ScreenDiagnostics {
    text_alignment: TextAlignment,
    diagnostics: BTreeMap<String, DiagnosticsText>,
    layout_changed: bool,
}

struct DiagnosticsText {
    id: DiagnosticId,
    agg: Aggregate,
    format: FormatFn,
    show_name: bool,
    colors: (Color, Color),
    edit: bool,
    rebuild: bool,
}

impl DiagnosticsText {
    fn format(&self, v: f64) -> String {
        let formatter = self.format;
        formatter(v)
    }
}

/// Builder-like interface for a [DiagnosticsText].
pub struct DiagnosticsTextBuilder<'a> {
    m: &'a mut BTreeMap<String, DiagnosticsText>,
    k: String,
}

impl<'a> DiagnosticsTextBuilder<'a> {
    /// Set the Aggregate function for this [DiagnosticsText]
    pub fn aggregate(self, agg: Aggregate) -> Self {
        self.m.entry(self.k.clone()).and_modify(|e| {
            e.agg = agg;
            e.rebuild = true;
        });
        self
    }

    /// Set the formatting function for this [DiagnosticsText]
    pub fn format(self, format: FormatFn) -> Self {
        self.m.entry(self.k.clone()).and_modify(|e| {
            e.format = format;
            dbg!(format);
            e.rebuild = true;
        });
        self
    }

    /// Set the text color for the diagnostic value
    pub fn diagnostic_color(self, color: Color) -> Self {
        self.m.entry(self.k.clone()).and_modify(|e| {
            e.colors.0 = color;
            e.edit = true;
        });
        self
    }

    /// Set the text color for the diagnostic name
    pub fn name_color(self, color: Color) -> Self {
        self.m.entry(self.k.clone()).and_modify(|e| {
            e.colors.1 = color;
            e.edit = true;
        });
        self
    }

    /// Toggle whhether the diagnostic name is displayed.
    pub fn toggle_name(self) -> Self {
        self.m.entry(self.k.clone()).and_modify(|e| {
            e.show_name = !e.show_name;
            e.edit = true;
        });
        self
    }
}

impl ScreenDiagnostics {
    /// Add a diagnostic to be displayed.
    ///
    /// * `name` - The name displayed on-screen. Also used as a key.
    /// * `diagnostic` - The [DiagnosticId] which is displayed.

    /// ```rust
    /// screen_diagnostics
    ///   .add(
    ///     "ms/frame".to_string(),
    ///     FrameTimeDiagnosticsPlugin::FRAME_TIME,
    ///   )
    ///   .aggregate(Aggregate::Value)
    ///   .format(|v| format!("{:.0}", v));
    /// ```
    pub fn add<S>(&mut self, name: S, id: DiagnosticId) -> DiagnosticsTextBuilder
    where
        S: Into<String>,
    {
        let text = DiagnosticsText {
            id,
            agg: Aggregate::Value,
            format: |v| format!("{:.2}", v),
            show_name: true,
            colors: DEFAULT_COLORS,
            edit: false,
            rebuild: true,
        };
        let name: String = name.into();
        self.diagnostics.insert(name.clone(), text);

        DiagnosticsTextBuilder {
            m: &mut self.diagnostics,
            k: name,
        }
    }

    /// Modify a [DiagnosticsText] by name.
    ///
    /// Uses the same syntax as [add_text]
    pub fn modify<S>(&mut self, name: S) -> DiagnosticsTextBuilder
    where
        S: Into<String>,
    {
        DiagnosticsTextBuilder {
            m: &mut self.diagnostics,
            k: name.into(),
        }
    }

    /// Remove a diagnostic by name.
    #[allow(dead_code)]
    pub fn remove(&mut self, name: String) {
        self.diagnostics.remove(&name);
    }

    /// Set the [TextAlignment] and trigger a rebuild
    pub fn set_alignment(&mut self, align: TextAlignment) {
        self.text_alignment = align;
        self.layout_changed = true;
    }

    fn update(&mut self, diagnostics: Res<Diagnostics>, mut text: Mut<Text>) {
        if self.layout_changed {
            return;
        }

        for (i, (key, mut text_diag)) in self.diagnostics.iter_mut().rev().enumerate() {
            if text_diag.rebuild {
                self.layout_changed = true;
                text_diag.rebuild = false;
                continue;
            }
            if text_diag.edit {
                text.sections[i * 2].style.color = text_diag.colors.0;
                text.sections[(i * 2) + 1].style.color = text_diag.colors.1;
                if text_diag.show_name {
                    text.sections[(i * 2) + 1].value = format!(" {} ", key);
                } else {
                    text.sections[(i * 2) + 1].value = " ".to_string();
                }
                text_diag.edit = false;
            }

            if let Some(diag) = diagnostics.get(text_diag.id) {
                let diag_val = match text_diag.agg {
                    Aggregate::Value => diag.value(),
                    Aggregate::Average => diag.average(),
                    Aggregate::MovingAverage(count) => {
                        let skip_maybe = diag.history_len().checked_sub(count);
                        skip_maybe.map(|skip| diag.values().skip(skip).sum::<f64>() / count as f64)
                    }
                };
                if let Some(val) = diag_val {
                    text.sections[i * 2].value = text_diag.format(val);
                }
            }
        }
    }

    fn rebuild(&mut self, font: Res<ScreenDiagnosticsFont>) -> Text {
        let mut sections: Vec<TextSection> = Vec::with_capacity(self.diagnostics.len());
        for (name, text) in self.diagnostics.iter().rev() {
            sections.append(&mut self.section(name, font.0.clone(), text));
        }

        Text {
            sections,
            alignment: self.text_alignment,
        }
    }

    fn section(
        &self,
        name: &str,
        font: Handle<Font>,
        textdiag: &DiagnosticsText,
    ) -> Vec<TextSection> {
        let text = if textdiag.show_name {
            format!(" {} ", name)
        } else {
            " ".to_string()
        };
        vec![
            TextSection {
                value: "".to_string(),
                style: TextStyle {
                    font: font.clone(),
                    font_size: 20.0,
                    color: textdiag.colors.0,
                },
            },
            TextSection {
                value: text,
                style: TextStyle {
                    font,
                    font_size: 20.0,
                    color: textdiag.colors.1,
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
    mut diags: ResMut<ScreenDiagnostics>,
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
    mut diag: ResMut<ScreenDiagnostics>,
    diagnostics: Res<Diagnostics>,
    mut query: Query<&mut Text, With<DiagnosticsTextMarker>>,
) {
    let text = query.single_mut();
    diag.update(diagnostics, text);
}
