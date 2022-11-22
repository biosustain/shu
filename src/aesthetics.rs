use crate::escher::{ArrowTag, CircleTag};
use crate::funcplot::{geom_scale, max_f32, min_f32, plot_hist, plot_kde};
use crate::geom::{GeomArrow, GeomHist, GeomMetabolite, Side};
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
            .add_system(plot_side_hist)
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
                } else {
                    options.line_width = 10.;
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

fn lerp_hsv(t: f32) -> Color {
    let mut t = t;
    let mut a = ColorHsl {
        h: 114.,
        s: 0.2,
        v: 0.7,
    };
    let mut b = ColorHsl {
        h: 114.,
        s: 0.8,
        v: 0.7,
    };

    // Hue interpolation
    let mut d = b.h - a.h;
    let h: f32;
    if a.h > b.h {
        (b.h, a.h) = (b.h, a.h);
        d = -d;
        t = 1. - t;
    }
    if d > 180. {
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
                } else {
                    *color = Color::rgb(0.85, 0.85, 0.85);
                }
            }
        }
    }
}

/// Plot size as numerical variable in metabolic circles.
pub fn plot_metabolite_size(
    mut query: Query<(&mut Path, &CircleTag)>,
    mut aes_query: Query<(&Point<f32>, &Aesthetics), (With<GeomMetabolite>, With<Gsize>)>,
) {
    for (sizes, aes) in aes_query.iter_mut() {
        for (mut path, arrow) in query.iter_mut() {
            let radius = if let Some(index) = aes.identifiers.iter().position(|r| r == &arrow.id) {
                sizes.0[index]
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
                    *color = lerp_hsv((colors.0[index] - min_val) / (max_val - min_val));
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
    mut aes_query: Query<(&Distribution<f32>, &Aesthetics, &GeomHist), With<Gy>>,
) {
    for (dist, aes, geom) in aes_query.iter_mut() {
        for (trans, arrow, path) in query.iter_mut() {
            if let Some(index) = aes.identifiers.iter().position(|r| r == &arrow.id) {
                let line = plot_hist(&dist.0[index], 6);
                let mut transform =
                    Transform::from_xyz(trans.translation.x, trans.translation.y, 0.5);
                match geom.side {
                    Side::Left => {
                        transform.rotate(Quat::from_rotation_z(-std::f32::consts::PI / 2.))
                    }
                    Side::Right => {
                        transform.rotate(Quat::from_rotation_z(std::f32::consts::PI / 2.))
                    }
                };
                let scale = geom_scale(path, &line);
                transform.scale.x *= scale;

                commands
                    .spawn(GeometryBuilder::build_as(
                        &line,
                        DrawMode::Fill(FillMode::color(Color::hex("7dce96").unwrap())),
                        transform,
                    ))
                    // this will remove them the next time side reaction is loaded
                    .insert((*geom).clone());
            }
        }
    }
}

/// Normalize the height of histograms to be comparable with each other.
fn normalize_histogram_height(mut query: Query<(&mut Transform, &Path, &GeomHist)>) {
    let max = max_f32(
        &query
            .iter()
            .filter_map(|(_, path, geom)| match geom.side {
                Side::Left => Some(path),
                Side::Right => None,
            })
            .flat_map(|path| path.0.iter().map(|ev| ev.to().y))
            .collect::<Vec<f32>>(),
    );

    for (mut trans, path, geom) in query.iter_mut() {
        if let Side::Left = geom.side {
            let height = max_f32(&path.0.iter().map(|ev| ev.to().y).collect::<Vec<f32>>());
            trans.scale.y = max * 30. / height;
        }
    }
    let max = max_f32(
        &query
            .iter()
            .filter_map(|(_, path, geom)| match geom.side {
                Side::Right => Some(path),
                Side::Left => None,
            })
            .flat_map(|path| path.0.iter().map(|ev| ev.to().y))
            .collect::<Vec<f32>>(),
    );

    for (mut trans, path, geom) in query.iter_mut() {
        if let Side::Right = geom.side {
            let height = max_f32(&path.0.iter().map(|ev| ev.to().y).collect::<Vec<f32>>());
            trans.scale.y = max * 30. / height;
        }
    }
}
