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

/// A dirty way of plotting Kdes with Paths as an histogram
pub fn plot_kde(samples: &[f32], n: u32) -> Path {
    let mut path_builder = PathBuilder::new();
    let center = samples.iter().sum::<f32>() / samples.len() as f32;

    for x in linspace(min_f32(samples), max_f32(samples), n) {
        let y = kde(x, samples, 1.06);
        info!("Path on {}", y);
        path_builder.move_to(Vec2::new(x - center, 0.));
        path_builder.line_to(Vec2::new(x - center, y));
    }
    path_builder.build()
}

/// Histogram plotting with n bins
pub fn plot_hist(samples: &[f32], bins: u32) -> Path {
    let mut path_builder = PathBuilder::new();
    let center = samples.iter().sum::<f32>() / samples.len() as f32;
    let anchors = linspace(min_f32(samples), max_f32(samples), bins);

    for (a, b) in [0.]
        .iter()
        // TODO: remove this clone
        .chain(anchors.clone()[0..anchors.len() - 1].iter())
        .zip(anchors)
    {
        // TODO: sort first this and operate over indices
        let y = samples.iter().filter(|&&x| (x >= *a) & (x < b)).count();
        // draw a rectangle
        path_builder.move_to(Vec2::new(a - center, 0.));
        path_builder.line_to(Vec2::new(a - center, y as f32 * -10.));
        path_builder.line_to(Vec2::new(b - center, y as f32 * -10.));
        path_builder.line_to(Vec2::new(b - center, 0.));
    }
    path_builder.build()
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

pub fn geom_scale(path: &Path, path_to_scale: &Path) -> f32 {
    let first_point = Vec2::new(
        get_extreme(path, false, true),
        get_extreme(path, false, false),
    );
    let last_point = Vec2::new(
        get_extreme(path, true, true),
        get_extreme(path, true, false),
    );
    let first_local = Vec2::new(
        get_extreme(path_to_scale, false, true),
        get_extreme(path_to_scale, false, false),
    );
    let last_local = Vec2::new(
        get_extreme(path_to_scale, true, true),
        get_extreme(path_to_scale, true, false),
    );
    (last_point - first_point).length() / (last_local - first_local).length()
}