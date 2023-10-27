//! Functions for plotting data.

use bevy::prelude::{
    Color, Component, Font, Handle, Text, Text2dBundle, TextStyle, Transform, Vec2,
};
use bevy_prototype_lyon::{
    entity::ShapeBundle,
    prelude::{GeometryBuilder, Path, PathBuilder, Stroke},
    shapes,
};
use colorgrad::{Color as GradColor, CustomGradient, Gradient};

#[derive(Component)]
/// Marker trait to avoid outputting an [`Entity`] to the screen.
pub struct IgnoreSave;

/// Maximum of a slice.
pub fn max_f32(slice: &[f32]) -> f32 {
    slice
        .iter()
        .fold(0f32, |acc, x| if x - acc > 1e-8 { *x } else { acc })
}

/// Minimum of a slice.
pub fn min_f32(slice: &[f32]) -> f32 {
    slice
        .iter()
        .fold(0f32, |acc, x| if x - acc <= 1e-8 { *x } else { acc })
}

fn std_normal(x: f32) -> f32 {
    std::f32::consts::E.powf(-x.powi(2) / 2.) / (2. * std::f32::consts::PI).sqrt()
}

fn kde(x: f32, samples: &[f32], h: f32) -> f32 {
    1. / (h * samples.len() as f32)
        * samples
            .iter()
            .map(|x_i| std_normal((x - x_i) / h))
            .sum::<f32>()
}

pub fn linspace(start: f32, stop: f32, nstep: u32) -> Vec<f32> {
    let delta: f32 = (stop - start) / (nstep as f32 - 1.);
    (0..(nstep)).map(|i| start + i as f32 * delta).collect()
}

enum PlottingState {
    Zero,
    Over { last_x: f32 },
}

/// Plot a density with a normal kernel using [`Paths`].
///
/// The path defines a set of positive curves starting when `y_0 > 0` at `[x_0, y_0]`
/// to n consecutive `[x_n, y]` KDE evaluations until `y == 0` again. The last line
/// is `[x_n, 0]` -> `[x_0, 0]` and the path is closed.
///
/// This way, artifacts produced when tesselating infinitesimal areas or when the
/// path is not closed are avoided.
pub fn plot_kde(samples: &[f32], n: u32, size: f32, xlimits: (f32, f32)) -> Option<Path> {
    let center = size / 2.;
    let anchors = linspace(-center, center, n);
    if center.is_nan() {
        return None;
    }
    if samples.is_empty() {
        return None;
    }
    let mut path_builder = PathBuilder::new();
    if samples.len() == 1 {
        path_builder = plot_spike(path_builder, samples[0], xlimits, center);
    } else {
        let mut state = PlottingState::Zero;
        path_builder.move_to(Vec2::new(anchors[0], 0.));
        for (point_x, anchor_x) in linspace(xlimits.0, xlimits.1, n).iter().zip(anchors.iter()) {
            let y = f32::max(kde(*point_x, samples, 1.06), 0.);
            match state {
                PlottingState::Zero => {
                    if y > 0. {
                        path_builder.move_to(Vec2::new(*anchor_x, y));
                        state = PlottingState::Over { last_x: *anchor_x };
                    }
                }
                PlottingState::Over { last_x } => {
                    path_builder.line_to(Vec2::new(*anchor_x, y));
                    if y == 0. {
                        path_builder.line_to(Vec2::new(last_x, 0.));
                        state = PlottingState::Zero;
                    }
                }
            }
        }
        if let PlottingState::Over { last_x } = state {
            path_builder.line_to(Vec2::new(anchors[anchors.len() - 1], 0.));
            path_builder.line_to(Vec2::new(last_x, 0.));
        }
    }
    Some(path_builder.build())
}

/// Histogram plotting with n bins.
pub fn plot_hist(samples: &[f32], bins: u32, size: f32, xlimits: (f32, f32)) -> Option<Path> {
    let center = size / 2.;
    // a bin should not be less than a data point
    let bins = u32::min(samples.len() as u32 / 2, bins);
    // actual x points to be mapped to the KDE
    let points = linspace(xlimits.0, xlimits.1, bins);
    // calculated x positions in the graph
    let anchors = linspace(-center, center, bins);
    if center.is_nan() {
        return None;
    }
    if samples.is_empty() {
        return None;
    }

    let mut path_builder = PathBuilder::new();
    if samples.len() == 1 {
        path_builder = plot_spike(path_builder, samples[0], xlimits, center);
    } else {
        for ((anchor_a, anchor_b), (point_a, point_b)) in anchors.clone()[0..(anchors.len() - 1)]
            .iter()
            .zip(anchors[1..anchors.len()].iter())
            .zip(
                [0.].iter()
                    .chain(points.clone()[0..(points.len() - 1)].iter())
                    .zip(points[1..points.len()].iter()),
            )
        {
            // TODO: sort first this and operate over indices
            let y = samples
                .iter()
                .filter(|&&x| (x >= *point_a) & (x < *point_b))
                .count();
            if y == 0 {
                continue;
            }
            path_builder.move_to(Vec2::new(*anchor_a, 0.));
            path_builder.line_to(Vec2::new(*anchor_a, y as f32));
            path_builder.line_to(Vec2::new(*anchor_b, y as f32));
            path_builder.line_to(Vec2::new(*anchor_b, 0.));
        }
    }
    Some(path_builder.build())
}

fn plot_spike(
    mut path_builder: PathBuilder,
    t: f32,
    xlimits: (f32, f32),
    center: f32,
) -> PathBuilder {
    let x = lerp(t, xlimits.0, xlimits.1, -center, center);
    // TODO: not clear how big this should be
    const EPS: f32 = 2.0;

    path_builder.move_to(Vec2::new(x - EPS, 0.));
    path_builder.line_to(Vec2::new(x - EPS, 1.0));
    path_builder.line_to(Vec2::new(x + EPS, 1.0));
    path_builder.line_to(Vec2::new(x + EPS, 0.));
    path_builder
}

/// Plot a box where the color is the mean of the samples.
pub fn plot_box_point(n_cond: usize, cond_index: usize) -> Path {
    let box_size = 40.;
    let box_center = if n_cond == 0 {
        0.
    } else {
        let center = cond_index as f32 * box_size * 1.2;
        center - n_cond as f32 * box_size * 1.2 / 2.
    };
    let mut path_builder = PathBuilder::new();
    path_builder.move_to(Vec2::new(box_center - box_size / 2., 0.));
    path_builder.line_to(Vec2::new(box_center + box_size / 2., 0.));
    path_builder.line_to(Vec2::new(box_center + box_size / 2., box_size));
    path_builder.line_to(Vec2::new(box_center - box_size / 2., box_size));
    path_builder.line_to(Vec2::new(box_center - box_size / 2., 0.));
    path_builder.build()
}

/// Bundle for text that goes into plot scales.
#[derive(Clone)]
pub struct ScaleBundle {
    pub x_0: Text2dBundle,
    pub y: Text2dBundle,
    pub x_n: Text2dBundle,
}

impl ScaleBundle {
    /// Build text components from minimum, maximum and mean values.
    pub fn new(
        minimum: f32,
        maximum: f32,
        mean: f32,
        mean_pos: f32,
        size: f32,
        font: Handle<Font>,
        font_size: f32,
        color: Color,
    ) -> Self {
        // build x component
        let x_0 = Text2dBundle {
            text: Text::from_section(
                format!("{:+.3e}", minimum),
                TextStyle {
                    font: font.clone(),
                    font_size,
                    color,
                },
            ),
            // to the left so that it is centered
            transform: Transform::from_xyz(-size / 2. - font_size * 2., 0., 0.2),
            ..Default::default()
        };
        let x_n = Text2dBundle {
            text: Text::from_section(
                format!("{:+.3e}", maximum),
                TextStyle {
                    font: font.clone(),
                    font_size,
                    color,
                },
            ),
            transform: Transform::from_xyz(size / 2., 0., 0.2),
            ..Default::default()
        };
        let y = Text2dBundle {
            text: Text::from_section(
                format!("{:+.3e}", mean),
                TextStyle {
                    font,
                    font_size,
                    color,
                },
            ),
            transform: Transform::from_xyz(mean_pos, 0., 0.2),
            ..Default::default()
        };
        Self { x_0, y, x_n }
    }
}

pub fn plot_line(size: f32, transform: Transform) -> (ShapeBundle, Stroke) {
    let mut path_builder = PathBuilder::new();
    path_builder.move_to(Vec2::new(-size / 2., 0.));
    path_builder.line_to(Vec2::new(size / 2., 0.));
    (
        ShapeBundle {
            path: GeometryBuilder::build_as(&path_builder.build()),
            visibility: bevy::prelude::Visibility::Hidden,
            transform,
            ..Default::default()
        },
        Stroke::color(Color::BLACK),
    )
}

/// Build and position text tags to indicate the scale of thethe  x-axis.
pub fn plot_scales(samples: &[f32], size: f32, font: Handle<Font>, font_size: f32) -> ScaleBundle {
    let mean: f32 = samples.iter().sum::<f32>() / samples.len() as f32;
    let min = min_f32(samples);
    let max = max_f32(samples);
    let mean_pos = lerp(mean, min, max, -size / 2., size / 2.);
    ScaleBundle::new(
        min,
        max,
        mean,
        mean_pos,
        size,
        font,
        font_size,
        Color::rgb(51. / 255., 78. / 255., 107. / 255.),
    )
}

fn get_extreme(path: &Path, maximum: bool, x: bool) -> f32 {
    let vec = &path
        .0
        .iter()
        .map(|p| if x { p.to().x } else { p.to().y })
        .chain(
            path.0
                .iter()
                .map(|p| if x { p.from().x } else { p.from().y }),
        )
        .collect::<Vec<f32>>();
    if maximum {
        max_f32(vec)
    } else {
        min_f32(vec)
    }
}

/// Get the size of a path as the largest distance between its points.
pub fn path_to_vec(path: &Path) -> Vec2 {
    let first_point = Vec2::new(
        get_extreme(path, false, true),
        get_extreme(path, false, false),
    );
    let last_point = Vec2::new(
        get_extreme(path, true, true),
        get_extreme(path, true, false),
    );
    last_point - first_point
}

/// Interpolate a value `t` in domain `[min_1, max_1]` to `[min_2, max_2]`.
pub fn lerp(t: f32, min_1: f32, max_1: f32, min_2: f32, max_2: f32) -> f32 {
    // clamp min and max to avoid explosion with low values on the first domain
    if t >= max_1 {
        max_2
    } else if t <= min_1 {
        min_2
    } else {
        (t - min_1) / (max_1 - min_1) * (max_2 - min_2) + min_2
    }
}

/// Three point interpolation, with 0 as middle point.
pub fn zero_lerp(t: f32, min_1: f32, max_1: f32, min_2: f32, max_2: f32) -> f32 {
    let (t, min_1, max_1) = if (min_1 * max_1) > 0. {
        (t, min_1, max_1)
    } else if t > 0. {
        (t, 0., max_1)
    } else {
        (t.abs(), 0., min_1.abs())
    };
    lerp(t, min_1, max_1, min_2, max_2)
}

fn to_grad(col: &bevy_egui::egui::Rgba) -> GradColor {
    GradColor::from_linear_rgba(
        col.r() as f64,
        col.g() as f64,
        col.b() as f64,
        col.a() as f64,
    )
}

/// Get the color for a given `t` from a `Gradient` with clamping to avoid exploding when the domain is very low.
pub fn from_grad_clamped(grad: &Gradient, t: f32, min_val: f32, max_val: f32) -> Color {
    let t = f32::clamp(t, min_val, max_val) as f64;
    let rgba = grad.at(t).to_linear_rgba();
    Color::rgba(rgba.0 as f32, rgba.1 as f32, rgba.2 as f32, rgba.3 as f32)
}

/// Build a `Gradient` for color interpolation between two colors from
/// the domain defined by [min_val, max_val] or [min_val, 0) [0, max_val]
/// if `zero` is `true`.
pub fn build_grad(
    zero: bool,
    min_val: f32,
    max_val: f32,
    min_color: &bevy_egui::egui::Rgba,
    max_color: &bevy_egui::egui::Rgba,
) -> colorgrad::Gradient {
    let mut grad = CustomGradient::new();
    if zero & ((min_val * max_val) < 0.) {
        grad.colors(&[
            to_grad(min_color),
            to_grad(&bevy_egui::egui::Rgba::from_rgb(0.83, 0.83, 0.89)),
            to_grad(max_color),
        ])
        .domain(&[min_val as f64, 0., max_val as f64])
    } else {
        grad.colors(&[to_grad(min_color), to_grad(max_color)])
            .domain(&[min_val as f64, max_val as f64])
    }
    .mode(colorgrad::BlendMode::Oklab)
    .interpolation(colorgrad::Interpolation::CatmullRom)
    .build()
    .expect("no gradient")
}

pub fn draw_arrow(from: Vec2, to: Vec2, offset: f32) -> shapes::Circle {
    // with an offset to avoid being hidden by metabolites
    let u = (to - from) / (to - from).length();
    let to = to - offset * u;
    shapes::Circle {
        radius: 5.0,
        center: to,
    }
}
