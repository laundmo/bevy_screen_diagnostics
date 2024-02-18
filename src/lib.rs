#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
#![warn(missing_docs)]
#![warn(rustdoc::missing_doc_code_examples)]

use std::{collections::BTreeMap, time::Duration};

use bevy::{
    diagnostic::{DiagnosticPath, DiagnosticsStore},
    prelude::*,
    render::view::RenderLayers,
    text::BreakLineOn,
    time::common_conditions::on_timer,
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
    ///# use bevy::prelude::*;
    ///# use bevy_screen_diagnostics::ScreenDiagnosticsPlugin;
    ///
    ///# fn main() {
    ///#     App::new()
    ///#         .add_plugins(DefaultPlugins)
    ///#         .add_plugins(ScreenDiagnosticsPlugin {
    ///#             style:
    /// Style {
    ///     align_self: AlignSelf::FlexEnd,
    ///     position_type: PositionType::Absolute,
    ///     bottom: Val::Px(5.0),
    ///     right: Val::Px(15.0),
    ///     ..default()
    /// },
    ///#        ..default()
    ///#    });
    ///# }
    /// ```
    pub style: Style,
    /// The font used for the text. By default [FiraCodeBold](https://github.com/tonsky/FiraCode) is used.
    pub font: Option<&'static str>,
    /// The render layer for the UI
    pub render_layer: RenderLayers,
}

const DEFAULT_COLORS: (Color, Color) = (Color::RED, Color::WHITE);

impl Default for ScreenDiagnosticsPlugin {
    fn default() -> Self {
        Self {
            timestep: TIMESTEP_10_PER_SECOND,
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                bottom: Val::Px(5.0),
                right: Val::Px(15.0),
                ..default()
            },
            font: None,
            render_layer: RenderLayers::default(),
        }
    }
}

#[derive(Resource)]
struct FontOption(Option<&'static str>);

#[derive(Resource)]
struct DiagnosticsStyle(Style);

#[derive(Resource, Deref)]
struct DiagnosticsLayer(RenderLayers);

impl Plugin for ScreenDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScreenDiagnostics>()
            .insert_resource(FontOption(self.font))
            .init_resource::<ScreenDiagnosticsFont>()
            .insert_resource(DiagnosticsStyle(self.style.clone()))
            .insert_resource(DiagnosticsLayer(self.render_layer))
            .add_systems(Startup, spawn_ui)
            .add_systems(Update, update_onscreen_diags_layout)
            .add_systems(
                Update,
                update_diags.run_if(on_timer(Duration::from_secs_f64(self.timestep))),
            );
    }
}

#[derive(Resource)]
struct ScreenDiagnosticsFont(Handle<Font>);

impl FromWorld for ScreenDiagnosticsFont {
    fn from_world(world: &mut World) -> Self {
        let font = world.get_resource::<FontOption>().unwrap();
        let assets = world.get_resource::<AssetServer>().unwrap();
        let font = match font.0 {
            Some(font) => assets.load(font),
            #[cfg(not(feature = "builtin-font"))]
            None => panic!(
                "No default font supplied, please either set the `builtin-font` \
                 feature or provide your own font file by setting the `font` field of \
                 `ScreenDiagnosticsPlugin` to `Some(\"font_asset_path\")`"
            ),
            #[cfg(feature = "builtin-font")]
            None => Default::default(),
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

/// Resource which maps the name to the [DiagnosticPath], [Aggregate] and [ConvertFn]
#[derive(Resource)]
pub struct ScreenDiagnostics {
    text_alignment: JustifyText,
    diagnostics: BTreeMap<String, DiagnosticsText>,
    layout_changed: bool,
}
impl Default for ScreenDiagnostics {
    fn default() -> Self {
        Self {
            text_alignment: JustifyText::Left,
            diagnostics: Default::default(),
            layout_changed: Default::default(),
        }
    }
}

struct DiagnosticsText {
    name: String,
    path: DiagnosticPath,
    agg: Aggregate,
    format: FormatFn,
    show: bool,
    show_name: bool,
    colors: (Color, Color),
    edit: bool,
    rebuild: bool,
    index: Option<usize>,
}

impl DiagnosticsText {
    fn format(&self, v: f64) -> String {
        let formatter = self.format;
        formatter(v)
    }

    fn get_name(&self) -> String {
        match self.show_name {
            true => format!(" {} ", self.name),
            false => " ".to_string(),
        }
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

    /// Toggle whether the diagnostic name is displayed.
    pub fn toggle_name(self) -> Self {
        self.m.entry(self.k.clone()).and_modify(|e| {
            e.show_name = !e.show_name;
            e.edit = true;
        });
        self
    }

    /// Toggle whether the diagnostic is displayed at all.
    pub fn toggle(self) -> Self {
        self.m.entry(self.k.clone()).and_modify(|e| {
            e.show = !e.show;
            e.rebuild = true;
        });
        self
    }
}

impl ScreenDiagnostics {
    /// Add a diagnostic to be displayed.
    ///
    /// * `name` - The name displayed on-screen. Also used as a key.
    /// * `path` - The [DiagnosticPath] which is displayed.

    /// ```rust
    ///# use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};
    ///# use bevy_screen_diagnostics::{Aggregate, ScreenDiagnosticsPlugin,ScreenDiagnostics};
    ///
    ///# fn main() {
    ///#     App::new()
    ///#         .add_plugins(DefaultPlugins)
    ///#         .add_plugins(ScreenDiagnosticsPlugin::default())
    ///#         .add_systems(Startup, setup);
    ///# }
    /// fn setup(mut screen_diagnostics: ResMut<ScreenDiagnostics>) {
    ///     screen_diagnostics
    ///         .add(
    ///             "ms/frame".to_string(),
    ///             FrameTimeDiagnosticsPlugin::FRAME_TIME,
    ///         )
    ///         .aggregate(Aggregate::Value)
    ///         .format(|v| format!("{:.0}", v));
    /// }
    /// ```
    pub fn add<S>(&mut self, name: S, path: DiagnosticPath) -> DiagnosticsTextBuilder
    where
        S: Into<String>,
    {
        let name: String = name.into();

        let text = DiagnosticsText {
            name: name.clone(),
            path,
            agg: Aggregate::Value,
            format: |v| format!("{v:.2}"),
            show: true,
            show_name: true,
            colors: DEFAULT_COLORS,
            edit: false,
            rebuild: true,
            index: None,
        };

        self.diagnostics.insert(name.clone(), text);

        DiagnosticsTextBuilder {
            m: &mut self.diagnostics,
            k: name,
        }
    }

    /// Modify a [DiagnosticsText] by name.
    ///
    /// Uses the same syntax as [ScreenDiagnostics::add]
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

    /// Set the [JustifyText] and trigger a rebuild
    pub fn set_alignment(&mut self, align: JustifyText) {
        self.text_alignment = align;
        self.layout_changed = true;
    }

    fn update(&mut self, diagnostics: Res<DiagnosticsStore>, mut text: Mut<Text>) {
        if self.layout_changed {
            return;
        }

        for text_diag in self.diagnostics.values_mut().rev() {
            if text_diag.rebuild {
                self.layout_changed = true;
                text_diag.rebuild = false;
                continue;
            }

            // needs to be checked here so layout_changed is triggered
            if !text_diag.show {
                continue;
            }

            if text_diag.edit && text_diag.index.is_some() {
                // set the value color
                text.sections[text_diag.index.unwrap()].style.color = text_diag.colors.0;

                let name_t = &mut text.sections[text_diag.index.unwrap() + 1];

                // set the name color
                name_t.style.color = text_diag.colors.1;

                // toggle the name visibility
                name_t.value = text_diag.get_name();

                text_diag.edit = false;
            }

            if let Some(diag) = diagnostics.get(&text_diag.path) {
                let diag_val = match text_diag.agg {
                    Aggregate::Value => diag.value(),
                    Aggregate::Average => diag.average(),
                    Aggregate::MovingAverage(count) => {
                        let skip_maybe = diag.history_len().checked_sub(count);
                        skip_maybe.map(|skip| diag.values().skip(skip).sum::<f64>() / count as f64)
                    }
                };

                if let Some(val) = diag_val {
                    text.sections[text_diag.index.unwrap()].value = text_diag.format(val);
                }
            }
        }
    }

    fn rebuild(&mut self, font: Res<ScreenDiagnosticsFont>) -> Text {
        let mut sections: Vec<TextSection> = Vec::new();

        for (i, text) in self
            .diagnostics
            .values_mut()
            .rev()
            .filter(|t| t.show)
            .enumerate()
        {
            text.index = Some(i * 2);
            sections.append(&mut Self::section(font.0.clone(), text));
        }

        Text {
            sections,
            justify: self.text_alignment,
            linebreak_behavior: BreakLineOn::WordBoundary,
        }
    }

    fn section(font: Handle<Font>, textdiag: &DiagnosticsText) -> Vec<TextSection> {
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
                value: textdiag.get_name(),
                style: TextStyle {
                    font,
                    font_size: 20.0,
                    color: textdiag.colors.1,
                },
            },
        ]
    }
}

fn spawn_ui(
    mut commands: Commands,
    diag_style: Res<DiagnosticsStyle>,
    diag_layer: Res<DiagnosticsLayer>,
) {
    commands
        .spawn((
            TextBundle {
                style: diag_style.0.clone(),
                text: Text {
                    sections: vec![],
                    ..default()
                },
                ..default()
            },
            **diag_layer,
        ))
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
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<DiagnosticsTextMarker>>,
) {
    let text = query.single_mut();
    diag.update(diagnostics, text);
}
