use crate::escher::ArrowTag;
use crate::geom::GeomArrow;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::{DrawMode, StrokeMode};

pub struct AesPlugin;

impl Plugin for AesPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(plot_arrow_size)
            .add_system(plot_arrow_color)
            .add_system(plot_arrow_size_dist);
    }
}

#[derive(Component)]
pub struct Aesthetics {
    // flag to filter out the plotting
    // it will be moved to the Geoms since more than one group of Aes
    // can be a plotted with different geoms.
    pub plotted: bool,
    // ordered identifers that each aesthetic will be plotted at
    pub identifiers: Vec<String>,
}

#[derive(Component)]
pub struct Gx {}

#[derive(Component)]
struct Gy {}

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
    mut query: Query<(&mut DrawMode, &ArrowTag)>,
    mut aes_query: Query<(&Point<f32>, &Aesthetics), (With<GeomArrow>, With<Gsize>)>,
) {
    for (sizes, aes) in aes_query.iter_mut() {
        for (mut draw_mode, arrow) in query.iter_mut() {
            if let DrawMode::Stroke(StrokeMode {
                ref mut options, ..
            }) = *draw_mode
            {
                if let Some(index) = aes.identifiers.iter().position(|r| r == &arrow.id) {
                    options.line_width = sizes.0[index];
                }
            }
        }
    }
}

/// For arrows (reactions) sizes, distributions are summarised as the mean.
pub fn plot_arrow_size_dist(
    mut query: Query<(&mut DrawMode, &ArrowTag)>,
    mut aes_query: Query<(&Distribution<f32>, &Aesthetics), (With<GeomArrow>, With<Gsize>)>,
) {
    for (sizes, aes) in aes_query.iter_mut() {
        for (mut draw_mode, arrow) in query.iter_mut() {
            if let DrawMode::Stroke(StrokeMode {
                ref mut options, ..
            }) = *draw_mode
            {
                if let Some(index) = aes.identifiers.iter().position(|r| r == &arrow.id) {
                    options.line_width =
                        sizes.0[index].iter().sum::<f32>() / sizes.0[index].len() as f32;
                }
            }
        }
    }
}

fn max_f32(slice: &[f32]) -> f32 {
    slice
        .iter()
        .fold(0f32, |acc, x| if x - acc > 1e-5 { *x } else { acc })
}
fn min_f32(slice: &[f32]) -> f32 {
    slice
        .iter()
        .fold(0f32, |acc, x| if x - acc <= 1e-5 { *x } else { acc })
}

struct ColorHsl {
    h: f32,
    s: f32,
    v: f32,
}

fn lerp_hsv(t: f32) -> Color {
    let mut t = t;
    let mut a = ColorHsl {
        h: 10.,
        s: 0.2,
        v: 0.4,
    };
    let mut b = ColorHsl {
        h: 180.,
        s: 0.8,
        v: 0.8,
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
        a.h = a.h + 360.; // 360deg
        h = (a.h + t * (b.h - a.h)) % 360.; // 360deg
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

/// Plot Color as numerical variable in arrows.
pub fn plot_arrow_color(
    mut query: Query<(&mut DrawMode, &ArrowTag)>,
    mut aes_query: Query<(&Point<f32>, &Aesthetics), (With<GeomArrow>, With<Gcolor>)>,
) {
    for (colors, aes) in aes_query.iter_mut() {
        let min_val = min_f32(&colors.0);
        let max_val = max_f32(&colors.0);
        for (mut draw_mode, arrow) in query.iter_mut() {
            if let DrawMode::Stroke(StrokeMode { ref mut color, .. }) = *draw_mode {
                if let Some(index) = aes.identifiers.iter().position(|r| r == &arrow.id) {
                    *color = lerp_hsv((colors.0[index] - min_val) / (max_val - min_val));
                }
            }
        }
    }
}
