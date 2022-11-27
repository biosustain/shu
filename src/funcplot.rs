use bevy::prelude::{info, Vec2};
use bevy_prototype_lyon::prelude::{Path, PathBuilder};

pub fn max_f32(slice: &[f32]) -> f32 {
    slice
        .iter()
        .fold(0f32, |acc, x| if x - acc > 1e-5 { *x } else { acc })
}

pub fn min_f32(slice: &[f32]) -> f32 {
    slice
        .iter()
        .fold(0f32, |acc, x| if x - acc <= 1e-5 { *x } else { acc })
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
    return (0..(nstep)).map(|i| start + i as f32 * delta).collect();
}

/// A dirty way of plotting Kdes with Paths.
pub fn plot_kde(samples: &[f32], n: u32, size: f32) -> Option<Path> {
    let center = size / 2.;
    let anchors = linspace(-center, size - center, n);
    let min_value = min_f32(samples);
    let max_value = max_f32(samples);
    if center.is_nan() {
        return None;
    }
    let mut path_builder = PathBuilder::new();
    path_builder.move_to(Vec2::new(-center, 0.));

    for (point_x, anchor_x) in linspace(min_value, max_value, n).iter().zip(anchors.iter()) {
        let y = kde(*point_x, samples, 1.06);
        path_builder.line_to(Vec2::new(*anchor_x, y));
    }
    path_builder.line_to(Vec2::new(size - center, 0.));
    path_builder.line_to(Vec2::new(-center, 0.));
    Some(path_builder.build())
}

/// Histogram plotting with n bins.
pub fn plot_hist(samples: &[f32], bins: u32, size: f32) -> Option<Path> {
    let center = size / 2.;
    // a bin should not be less than a data point
    let bins = u32::min(samples.len() as u32 / 2, bins);
    // actual x points to be mapped to the KDE
    let points = linspace(min_f32(samples), max_f32(samples), bins);
    // calculated x positions in the graph
    let anchors = linspace(-center, size - center, bins);
    if center.is_nan() {
        return None;
    }

    let mut path_builder = PathBuilder::new();
    for ((anchor_a, anchor_b), (point_a, point_b)) in anchors.clone()[0..anchors.len() - 1]
        .iter()
        .zip(anchors[1..anchors.len()].iter())
        .zip(
            [0.].iter()
                .chain(points.clone()[0..points.len() - 1].iter())
                .zip(points[1..points.len()].iter()),
        )
    {
        // TODO: sort first this and operate over indices
        let y = samples
            .iter()
            .filter(|&&x| (x >= *point_a) & (x < *point_b))
            .count();
        path_builder.move_to(Vec2::new(*anchor_a, 0.));
        path_builder.line_to(Vec2::new(*anchor_a, y as f32));
        path_builder.line_to(Vec2::new(*anchor_b, y as f32));
        path_builder.line_to(Vec2::new(*anchor_b, 0.));
    }
    Some(path_builder.build())
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

pub fn geom_scale(ref_path: &Path, path_to_scale: &Path) -> f32 {
    path_to_vec(ref_path).length() / path_to_vec(path_to_scale).length()
}

pub fn right_of_path(path: &Path) -> Vec2 {
    let reference_vec = path_to_vec(path);
    info!("reference: {}", reference_vec.normalize());
    let reference_vec = reference_vec.perp();
    info!("reference 90: {}", reference_vec.normalize());
    reference_vec
}
