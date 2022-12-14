//! Functions for plotting data.

use bevy::prelude::{Color, Font, Handle, Text, Text2dBundle, TextStyle, Transform, Vec2};
use bevy_prototype_lyon::prelude::{Path, PathBuilder};
use colorgrad::{Color as GradColor, CustomGradient, Gradient};

pub fn max_f32(slice: &[f32]) -> f32 {
    slice
        .iter()
        .fold(0f32, |acc, x| if x - acc > 1e-8 { *x } else { acc })
}

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

fn linspace(start: f32, stop: f32, nstep: u32) -> Vec<f32> {
    let delta: f32 = (stop - start) / (nstep as f32 - 1.);
    (0..(nstep)).map(|i| start + i as f32 * delta).collect()
}

/// A dirty way of plotting Kdes with Paths.
pub fn plot_kde(samples: &[f32], n: u32, size: f32, xlimits: (f32, f32)) -> Option<Path> {
    let center = size / 2.;
    let anchors = linspace(-center, size - center, n);
    if center.is_nan() {
        return None;
    }
    let mut path_builder = PathBuilder::new();
    path_builder.move_to(Vec2::new(-center, 0.));

    for (point_x, anchor_x) in linspace(xlimits.0, xlimits.1, n).iter().zip(anchors.iter()) {
        let y = kde(*point_x, samples, 1.06);
        path_builder.line_to(Vec2::new(*anchor_x, y));
    }
    path_builder.line_to(Vec2::new(size - center, 0.));
    path_builder.line_to(Vec2::new(-center, 0.));
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
    let anchors = linspace(-center, size - center, bins);
    if center.is_nan() {
        return None;
    }

    let mut path_builder = PathBuilder::new();
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
    Some(path_builder.build())
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
pub struct ScaleBundle {
    pub x_0: Text2dBundle,
    pub y: Text2dBundle,
    pub x_n: Text2dBundle,
}

/// Build and position text tags to indicate the scale of thethe  x-axis.
pub fn plot_scales(
    samples: &[f32],
    size: f32,
    font: Handle<Font>,
    font_size: f32,
    mean_y: f32,
) -> ScaleBundle {
    let mean: f32 = samples.iter().sum::<f32>() / samples.len() as f32;
    let min = min_f32(samples);
    let max = max_f32(samples);
    let mean_pos = lerp(mean, min, max, -size / 2., size / 2.);
    ScaleBundle {
        x_0: Text2dBundle {
            text: Text::from_section(
                format!("{:+.3e}", min),
                TextStyle {
                    font: font.clone(),
                    font_size,
                    color: Color::rgb(51. / 255., 78. / 255., 101. / 255.),
                },
            ),
            transform: Transform::from_xyz(-size / 2., 0., 0.2),
            ..Default::default()
        },
        y: Text2dBundle {
            text: Text::from_section(
                format!("{:+.3e}", mean),
                TextStyle {
                    font: font.clone(),
                    font_size,
                    color: Color::rgb(51. / 255., 78. / 255., 101. / 255.),
                },
            ),
            // the y axis seems to be OK here
            transform: Transform::from_xyz(mean_pos, mean_y, 0.2),
            ..Default::default()
        },
        x_n: Text2dBundle {
            text: Text::from_section(
                format!("{:+.3e}", max),
                TextStyle {
                    font,
                    font_size,
                    color: Color::rgb(51. / 255., 78. / 255., 101. / 255.),
                },
            ),
            transform: Transform::from_xyz(size / 2., 0., 0.2),
            ..Default::default()
        },
    }
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

/// Interpolat a value t in domain [min_1, max_1] to [min_2, max_2]
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

fn to_grad(col: &bevy_egui::egui::color::Rgba) -> GradColor {
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
    min_color: &bevy_egui::egui::color::Rgba,
    max_color: &bevy_egui::egui::color::Rgba,
) -> colorgrad::Gradient {
    let mut grad = CustomGradient::new();
    if zero & ((min_val * max_val) < 0.) {
        grad.colors(&[
            to_grad(min_color),
            to_grad(&bevy_egui::egui::color::Rgba::from_rgb(0.83, 0.83, 0.89)),
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
