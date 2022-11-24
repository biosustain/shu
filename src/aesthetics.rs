use crate::escher::{load_map, ArrowTag, CircleTag};
use crate::funcplot::{geom_scale, max_f32, min_f32, plot_hist, plot_kde, right_of_path};
use crate::geom::{GeomArrow, GeomHist, GeomMetabolite, HistTag, Side};
use crate::gui::UiState;
use bevy_egui::egui::epaint::color::Hsva;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::{
    shapes, DrawMode, FillMode, GeometryBuilder, Path, ShapePath, StrokeMode,
};

pub struct AesPlugin;

impl Plugin for AesPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(plot_arrow_size)
            .add_system(plot_arrow_color)
            .add_system(plot_arrow_size_dist)
            .add_system(plot_metabolite_color)
            .add_system(plot_side_hist.before(load_map))
            .add_system(normalize_histogram_height)
            .add_system(plot_metabolite_size);
    }
}

#[derive(Component)]
pub struct Aesthetics {
    /// flag to filter out the plotting
    /// it will be moved to the Geoms since more than one group of Aes
    /// can be a plotted with different geoms.
    pub plotted: bool,
    /// ordered identifers that each aesthetic will be plotted at
    pub identifiers: Vec<String>,
}

#[derive(Component)]
pub struct Gx {}

#[derive(Component)]
pub struct Gy {}

/// Data from the variables is allocated here.
#[derive(Component)]
pub struct Point<T>(pub Vec<T>);
#[derive(Component)]
pub struct Distribution<T>(pub Vec<Vec<T>>);
#[derive(Component)]
pub struct Categorical<T>(Vec<T>);

#[derive(Component)]
pub struct Gsize {}

#[derive(Component)]
pub struct Gcolor {}

/// Plot arrow size.
pub fn plot_arrow_size(
    ui_state: Res<UiState>,
    mut query: Query<(&mut DrawMode, &ArrowTag)>,
    mut aes_query: Query<(&Point<f32>, &Aesthetics), (With<GeomArrow>, With<Gsize>)>,
) {
    for (sizes, aes) in aes_query.iter_mut() {
        let min_val = min_f32(&sizes.0);
        let max_val = max_f32(&sizes.0);
        for (mut draw_mode, arrow) in query.iter_mut() {
            if let DrawMode::Stroke(StrokeMode {
                ref mut options, ..
            }) = *draw_mode
            {
                if let Some(index) = aes.identifiers.iter().position(|r| r == &arrow.id) {
                    let unscaled_width = sizes.0[index];
                    options.line_width = lerp(
                        unscaled_width,
                        min_val,
                        max_val,
                        ui_state.min_reaction,
                        ui_state.max_reaction,
                    );
                } else {
                    options.line_width = 10.;
                }
            }
        }
    }
}

/// For arrows (reactions) sizes, distributions are summarised as the mean.
pub fn plot_arrow_size_dist(
    ui_state: Res<UiState>,
    mut query: Query<(&mut DrawMode, &ArrowTag)>,
    mut aes_query: Query<(&Distribution<f32>, &Aesthetics), (With<GeomArrow>, With<Gsize>)>,
) {
    for (sizes, aes) in aes_query.iter_mut() {
        for (mut draw_mode, arrow) in query.iter_mut() {
            let min_val = min_f32(&sizes.0.iter().flatten().copied().collect::<Vec<f32>>());
            let max_val = max_f32(&sizes.0.iter().flatten().copied().collect::<Vec<f32>>());
            if let DrawMode::Stroke(StrokeMode {
                ref mut options, ..
            }) = *draw_mode
            {
                if let Some(index) = aes.identifiers.iter().position(|r| r == &arrow.id) {
                    let unscaled_width =
                        sizes.0[index].iter().sum::<f32>() / sizes.0[index].len() as f32;
                    options.line_width = lerp(
                        unscaled_width,
                        min_val,
                        max_val,
                        ui_state.min_reaction,
                        ui_state.max_reaction,
                    );
                } else {
                    options.line_width = 10.;
                }
            }
        }
    }
}

struct ColorHsl {
    h: f32,
    s: f32,
    v: f32,
}

fn lerp_hsv(t: f32, min_color: Hsva, max_color: Hsva) -> Color {
    let mut t = t;
    let mut a = ColorHsl {
        h: min_color.h,
        s: min_color.s,
        v: min_color.v,
    };
    let mut b = ColorHsl {
        h: max_color.h,
        s: max_color.s,
        v: max_color.v,
    };

    // Hue interpolation
    let mut d = b.h - a.h;
    let h: f32;
    if a.h > b.h {
        (b.h, a.h) = (b.h, a.h);
        d = -d;
        t = 1. - t;
    }
    if d > 0.5 {
        // 180deg
        a.h = a.h + 1.; // 360deg
        h = (a.h + t * (b.h - a.h)) % 1.; // 360deg
    } else {
        // 180deg
        h = a.h + t * d
    }
    // Interpolates the rest
    Color::hsl(
        h,                     // H
        a.s + t * (b.s - a.s), // S
        a.v + t * (b.v - a.v), // V
    )
}

fn lerp(t: f32, min_1: f32, max_1: f32, min_2: f32, max_2: f32) -> f32 {
    (t - min_1) / (max_1 - min_1) * (max_2 - min_2) + min_2
}

/// Plot Color as numerical variable in arrows.
pub fn plot_arrow_color(
    ui_state: Res<UiState>,
    mut query: Query<(&mut DrawMode, &ArrowTag)>,
    mut aes_query: Query<(&Point<f32>, &Aesthetics), (With<GeomArrow>, With<Gcolor>)>,
) {
    for (colors, aes) in aes_query.iter_mut() {
        let min_val = min_f32(&colors.0);
        let max_val = max_f32(&colors.0);
        for (mut draw_mode, arrow) in query.iter_mut() {
            if let DrawMode::Stroke(StrokeMode { ref mut color, .. }) = *draw_mode {
                if let Some(index) = aes.identifiers.iter().position(|r| r == &arrow.id) {
                    *color = lerp_hsv(
                        (colors.0[index] - min_val) / (max_val - min_val),
                        ui_state.min_reaction_color,
                        ui_state.max_reaction_color,
                    );
                } else {
                    *color = Color::rgb(0.85, 0.85, 0.85);
                }
            }
        }
    }
}

/// Plot size as numerical variable in metabolic circles.
pub fn plot_metabolite_size(
    ui_state: Res<UiState>,
    mut query: Query<(&mut Path, &CircleTag)>,
    mut aes_query: Query<(&Point<f32>, &Aesthetics), (With<GeomMetabolite>, With<Gsize>)>,
) {
    for (sizes, aes) in aes_query.iter_mut() {
        let min_val = min_f32(&sizes.0);
        let max_val = max_f32(&sizes.0);
        for (mut path, arrow) in query.iter_mut() {
            let radius = if let Some(index) = aes.identifiers.iter().position(|r| r == &arrow.id) {
                lerp(
                    sizes.0[index],
                    min_val,
                    max_val,
                    ui_state.min_metabolite,
                    ui_state.max_metabolite,
                )
            } else {
                20.
            };
            let polygon = shapes::RegularPolygon {
                sides: 6,
                feature: shapes::RegularPolygonFeature::Radius(radius),
                ..shapes::RegularPolygon::default()
            };
            *path = ShapePath::build_as(&polygon);
        }
    }
}

/// Plot Color as numerical variable in metabolic circles.
pub fn plot_metabolite_color(
    ui_state: Res<UiState>,
    mut query: Query<(&mut DrawMode, &CircleTag)>,
    mut aes_query: Query<(&Point<f32>, &Aesthetics), (With<GeomMetabolite>, With<Gcolor>)>,
) {
    for (colors, aes) in aes_query.iter_mut() {
        let min_val = min_f32(&colors.0);
        let max_val = max_f32(&colors.0);
        for (mut draw_mode, arrow) in query.iter_mut() {
            if let DrawMode::Outlined {
                fill_mode: FillMode { ref mut color, .. },
                ..
            } = *draw_mode
            {
                if let Some(index) = aes.identifiers.iter().position(|r| r == &arrow.id) {
                    *color = lerp_hsv(
                        (colors.0[index] - min_val) / (max_val - min_val),
                        ui_state.min_reaction_color,
                        ui_state.max_reaction_color,
                    );
                } else {
                    *color = Color::rgb(0.85, 0.85, 0.85);
                }
            }
        }
    }
}

/// Plot histogram as numerical variable next to arrows.
fn plot_side_hist(
    mut commands: Commands,
    mut query: Query<(&Transform, &ArrowTag, &Path)>,
    mut aes_query: Query<(&Distribution<f32>, &Aesthetics, &mut GeomHist), With<Gy>>,
) {
    for (dist, aes, mut geom) in aes_query.iter_mut() {
        if geom.rendered {
            continue;
        }
        for (trans, arrow, path) in query.iter_mut() {
            if let Some(index) = aes.identifiers.iter().position(|r| r == &arrow.id) {
                let this_dist = match dist.0.get(index) {
                    Some(d) => d,
                    None => continue,
                };
                let line = plot_hist(this_dist, 6);
                let (rotation_90, away, hex) = match geom.side {
                    Side::Right => (
                        -Vec2::Y.angle_between(arrow.direction.perp()),
                        -30.,
                        // TODO: this should be a setting
                        "7dce96",
                    ),
                    Side::Left => (
                        -Vec2::NEG_Y.angle_between(arrow.direction.perp()),
                        30.,
                        "DA9687",
                    ),
                };
                let mut transform =
                    Transform::from_xyz(trans.translation.x, trans.translation.y, 0.5)
                        .with_rotation(Quat::from_rotation_z(rotation_90));
                let scale = geom_scale(&path, &line);
                transform.scale.x *= scale;
                transform.translation.x += arrow.direction.perp().x * away;
                transform.translation.y += arrow.direction.perp().y * away;

                commands
                    .spawn(GeometryBuilder::build_as(
                        &line,
                        DrawMode::Fill(FillMode::color(Color::hex(hex).unwrap())),
                        transform,
                    ))
                    .insert(HistTag {
                        side: geom.side.clone(),
                    });
            }
        }
        geom.rendered = true;
    }
}

/// Normalize the height of histograms to be comparable with each other.
/// It treats the two sides independently.
fn normalize_histogram_height(
    ui_state: Res<UiState>,
    mut query: Query<(&mut Transform, &Path, &HistTag)>,
) {
    for (mut trans, path, hist) in query.iter_mut() {
        let height = max_f32(&path.0.iter().map(|ev| ev.to().y).collect::<Vec<f32>>());
        trans.scale.y = match hist.side {
            Side::Left => ui_state.max_left / height,
            Side::Right => ui_state.max_right / height,
        }
    }
}
