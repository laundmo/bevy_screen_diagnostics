#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
#![warn(missing_docs)]
#![warn(rustdoc::missing_doc_code_examples)]

use std::{collections::BTreeMap, time::Duration};

use bevy::color::palettes::css;
use bevy::{
    diagnostic::{DiagnosticPath, DiagnosticsStore},
    prelude::*,
    render::view::RenderLayers,
    text::LineBreak,
    time::common_conditions::on_timer,
};

mod extras;

#[cfg(feature = "sysinfo_plugin")]
pub use self::extras::sysinfo_plugin::ScreenSystemInformationDiagnosticsPlugin;
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
    /// Node {
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
    pub style: Node,
    /// The font used for the text. By default [FiraCodeBold](https://github.com/tonsky/FiraCode) is used.
    pub font: Option<&'static str>,
    /// The render layer for the UI
    pub render_layer: RenderLayers,
}

const DEFAULT_COLORS: (Srgba, Srgba) = (css::RED, css::WHITE);

impl Default for ScreenDiagnosticsPlugin {
    fn default() -> Self {
        Self {
            timestep: TIMESTEP_10_PER_SECOND,
            style: Node {
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

#[derive(Resource, Reflect)]
struct DiagnosticsStyle(Node);

#[derive(Resource, Deref, Reflect)]
struct DiagnosticsLayer(RenderLayers);

impl Plugin for ScreenDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScreenDiagnostics>()
            .insert_resource(FontOption(self.font))
            .init_resource::<ScreenDiagnosticsFont>()
            .insert_resource(DiagnosticsStyle(self.style.clone()))
            .insert_resource(DiagnosticsLayer(self.render_layer.clone()))
            .add_systems(Startup, spawn_ui)
            .add_systems(
                Update,
                (update_onscreen_diags_layout, update_diags)
                    .chain()
                    .run_if(on_timer(Duration::from_secs_f64(self.timestep))),
            );
    }
}

#[derive(Resource, Reflect)]
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

#[derive(Component, Reflect)]
#[require(Text)]
struct DiagnosticsTextMarker;

/// Aggregaes which can be used for displaying Diagnostics.
#[derive(Copy, Clone, Default, Debug, Reflect)]
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
#[derive(Resource, Reflect)]
#[reflect(from_reflect = false)]
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

// ngl, i still haven't fully grasped the various parts of bevy_reflect
// so i don't know if a placeholder like this as a default is useful
// hell, i dont even know if deriving Reflect on this at all will be useful...
const PLACEHOLDER_DIAGNOSTIC_PATH: DiagnosticPath =
    DiagnosticPath::const_new("bevy_screen_diagnostics/placeholder");
fn placeholder_path() -> DiagnosticPath {
    PLACEHOLDER_DIAGNOSTIC_PATH
}
fn placeholder_format() -> FormatFn {
    |v| format!("{v:.2}")
}

#[derive(Reflect)]
struct DiagnosticsText {
    name: String,
    // might not be useful to have reflect here at all, but i needed this to make it not complain
    #[reflect(ignore, default = "placeholder_path")]
    path: DiagnosticPath,
    agg: Aggregate,
    // might not be useful to have reflect here at all, but i needed this to make it not complain
    #[reflect(ignore, default = "placeholder_format")]
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

impl DiagnosticsTextBuilder<'_> {
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
    ///
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
            colors: (DEFAULT_COLORS.0.into(), DEFAULT_COLORS.1.into()),
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
}

fn spawn_ui(
    mut commands: Commands,
    diag_style: Res<DiagnosticsStyle>,
    diag_layer: Res<DiagnosticsLayer>,
) {
    commands.spawn((
        Text::default(),
        diag_style.0.clone(),
        diag_layer.clone(),
        DiagnosticsTextMarker,
    ));
}

fn update_onscreen_diags_layout(
    mut diags: ResMut<ScreenDiagnostics>,
    font: Res<ScreenDiagnosticsFont>,
    mut text_layout: Single<(Entity, &mut TextLayout), With<DiagnosticsTextMarker>>,
    mut commands: Commands,
) {
    if diags.layout_changed {
        commands.entity(text_layout.0).remove::<Children>();

        for (i, text) in diags
            .diagnostics
            .values_mut()
            .rev()
            .filter(|t| t.show)
            .enumerate()
        {
            text.index = Some(i * 2 + 1);
            commands.entity(text_layout.0).with_children(|c| {
                c.spawn((
                    TextSpan::new("test_val"),
                    TextFont::from_font(font.0.clone()).with_font_size(20.0),
                    TextColor(text.colors.0),
                ));
                c.spawn((
                    TextSpan::new(text.get_name()),
                    TextFont::from_font(font.0.clone()).with_font_size(20.0),
                    TextColor(text.colors.1),
                ));
            });
        }

        *text_layout.1 = TextLayout {
            justify: diags.text_alignment,
            linebreak: LineBreak::WordBoundary,
        };

        diags.layout_changed = false;
    }
}

fn update_diags(
    mut diag: ResMut<ScreenDiagnostics>,
    diagnostics: Res<DiagnosticsStore>,
    root_text: Single<Entity, With<DiagnosticsTextMarker>>,
    mut writer: TextUiWriter,
) -> Result {
    if diag.layout_changed {
        return Ok(());
    }
    let mut layout_changed = false;
    for text_diag in diag.diagnostics.values_mut().rev() {
        if text_diag.rebuild {
            layout_changed = true;
            text_diag.rebuild = false;
            continue;
        }
        // needs to be checked here otherwise this tries to edit bad texts
        if !text_diag.show {
            continue;
        }

        if let Some(index) = text_diag.index
            && text_diag.edit
        {
            // set the value color
            *writer.color(root_text.entity(), index) = text_diag.colors.0.into();
            // set the name color
            *writer.color(root_text.entity(), index + 1) = text_diag.colors.1.into();

            // toggle the name visibility
            *writer.text(root_text.entity(), index + 1) = text_diag.get_name();

            text_diag.edit = false;
        }

        if let Some(diag_val) = diagnostics.get(&text_diag.path) {
            let diag_val = match text_diag.agg {
                Aggregate::Value => diag_val.value(),
                Aggregate::Average => diag_val.average(),
                Aggregate::MovingAverage(count) => {
                    let skip_maybe = diag_val.history_len().checked_sub(count);
                    skip_maybe.map(|skip| diag_val.values().skip(skip).sum::<f64>() / count as f64)
                }
            };

            if let Some(val) = diag_val
                && let Some(index) = text_diag.index
            {
                *writer.text(root_text.entity(), index) = text_diag.format(val);
            }
        }
    }
    diag.layout_changed = layout_changed;
    Ok(())
}
