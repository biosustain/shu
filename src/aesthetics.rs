use crate::escher::{ArrowTag, CircleTag, Hover, Tag};
use crate::funcplot::{
    build_grad, from_grad_clamped, lerp, max_f32, min_f32, path_to_vec, plot_box_point, plot_hist,
    plot_kde, plot_line, plot_scales, zero_lerp,
};
use crate::geom::{
    AesFilter, AnyTag, Drag, GeomArrow, GeomHist, GeomMetabolite, HistPlot, HistTag, PopUp, Side,
    VisCondition, Xaxis,
};
use crate::gui::{or_color, UiState};
use itertools::Itertools;
use std::collections::HashMap;

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::{
    shapes, Fill, GeometryBuilder, Path, ShapeBundle, ShapePath, Stroke,
};

pub struct AesPlugin;

impl Plugin for AesPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RestoreEvent>()
            .add_systems(Update, plot_arrow_size)
            .add_systems(Update, plot_metabolite_size)
            .add_systems(Update, plot_arrow_color)
            .add_systems(Update, plot_metabolite_color)
            .add_systems(Update, restore_geoms::<CircleTag>)
            .add_systems(Update, restore_geoms::<ArrowTag>)
            .add_systems(Update, normalize_histogram_height)
            .add_systems(Update, unscale_histogram_children)
            .add_systems(Update, fill_conditions)
            .add_systems(Update, filter_histograms)
            .add_systems(Update, follow_the_axes)
            // TODO: check since these were before load_map
            .add_systems(Update, (build_axes, build_hover_axes, build_point_axes))
            .add_systems(Update, (plot_side_hist, plot_hover_hist))
            .add_systems(Update, (plot_side_box, change_color.before(plot_side_box)));
    }
}

#[derive(Component)]
pub struct Aesthetics {
    /// ordered identifers that each aesthetic will be plotted at
    pub identifiers: Vec<String>,
    /// ordered condition identifiers
    pub condition: Option<String>,
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

/// Marker to avoid scaling some Entities with HistTag.
#[derive(Component)]
pub struct Unscale;

/// Marker for things that need to change the color when UiChanges.
#[derive(Component)]
struct ColorListener {
    value: f32,
    min_val: f32,
    max_val: f32,
}

/// Everytime this is sent, all data and plots are removed, leaving
/// the map as default. This is triggered when new data is added.
#[derive(Event)]
pub struct RestoreEvent;

/// Plot arrow size.
pub fn plot_arrow_size(
    ui_state: Res<UiState>,
    mut query: Query<(&mut Stroke, &ArrowTag)>,
    mut aes_query: Query<(&Point<f32>, &Aesthetics, &GeomArrow), With<Gsize>>,
) {
    for (sizes, aes, _geom) in aes_query.iter_mut() {
        if let Some(condition) = &aes.condition {
            if condition != &ui_state.condition {
                continue;
            }
        }
        let min_val = min_f32(&sizes.0);
        let max_val = max_f32(&sizes.0);
        for (mut stroke, arrow) in query.iter_mut() {
            if let Some(index) = aes.identifiers.iter().position(|r| r == &arrow.id) {
                let unscaled_width = sizes.0[index];
                let f = if ui_state.zero_white { zero_lerp } else { lerp };
                stroke.options.line_width = f(
                    unscaled_width,
                    min_val,
                    max_val,
                    ui_state.min_reaction,
                    ui_state.max_reaction,
                );
            } else {
                stroke.options.line_width = 10.;
            }
        }
    }
}

/// Plot Color as numerical variable in circles.
pub fn plot_arrow_color(
    ui_state: Res<UiState>,
    mut query: Query<(&mut Stroke, &ArrowTag), Without<Fill>>,
    mut aes_query: Query<(&Point<f32>, &Aesthetics, &GeomArrow), With<Gcolor>>,
) {
    for (colors, aes, _) in aes_query.iter_mut() {
        if let Some(condition) = &aes.condition {
            if condition != &ui_state.condition {
                continue;
            }
        }
        let min_val = min_f32(&colors.0);
        let max_val = max_f32(&colors.0);
        let grad = build_grad(
            ui_state.zero_white,
            min_val,
            max_val,
            &ui_state.min_reaction_color,
            &ui_state.max_reaction_color,
        );
        for (mut stroke, tag) in query.iter_mut() {
            if let Some(index) = aes.identifiers.iter().position(|r| r == tag.id()) {
                stroke.color = from_grad_clamped(&grad, colors.0[index], min_val, max_val);
            } else {
                stroke.color = Color::rgb(0.85, 0.85, 0.85);
            }
        }
    }
}

/// Plot Color as numerical variable in Circles.
pub fn plot_metabolite_color(
    ui_state: Res<UiState>,
    mut query: Query<(&mut Fill, &CircleTag)>,
    mut aes_query: Query<(&Point<f32>, &Aesthetics, &GeomMetabolite), With<Gcolor>>,
) {
    for (colors, aes, _) in aes_query.iter_mut() {
        if let Some(condition) = &aes.condition {
            if condition != &ui_state.condition {
                continue;
            }
        }
        let min_val = min_f32(&colors.0);
        let max_val = max_f32(&colors.0);
        let grad = build_grad(
            ui_state.zero_white,
            min_val,
            max_val,
            &ui_state.min_metabolite_color,
            &ui_state.max_metabolite_color,
        );
        for (mut fill, tag) in query.iter_mut() {
            if let Some(index) = aes.identifiers.iter().position(|r| r == tag.id()) {
                fill.color = from_grad_clamped(&grad, colors.0[index], min_val, max_val);
            } else {
                fill.color = Color::rgb(0.85, 0.85, 0.85);
            }
        }
    }
}

/// Plot size as numerical variable in metabolic circles.
pub fn plot_metabolite_size(
    ui_state: Res<UiState>,
    mut query: Query<(&mut Path, &CircleTag)>,
    mut aes_query: Query<(&Point<f32>, &Aesthetics, &GeomArrow), With<Gsize>>,
) {
    for (sizes, aes, _geom) in aes_query.iter_mut() {
        if let Some(condition) = &aes.condition {
            if condition != &ui_state.condition {
                continue;
            }
        }
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

/// Remove colors and sizes from circles and arrows after new data is dropped.
fn restore_geoms<T: Tag>(
    mut restore_event: EventReader<RestoreEvent>,
    mut query: ParamSet<(
        Query<(&mut Fill, &mut Path), With<T>>,
        Query<&mut Stroke, (With<T>, Without<Fill>)>,
    )>,
) {
    for _ in restore_event.iter() {
        for (mut fill, mut path) in query.p0().iter_mut() {
            // met colors
            fill.color = T::default_color();
            let polygon = shapes::RegularPolygon {
                sides: 6,
                feature: shapes::RegularPolygonFeature::Radius(20.),
                ..shapes::RegularPolygon::default()
            };
            // met size
            *path = ShapePath::build_as(&polygon);
        }
        for mut stroke in query.p1().iter_mut() {
            stroke.color = T::default_color();
            stroke.options.line_width = 10.0;
        }
    }
}

/// Build axes for histograms, summarising all external information.
/// Each Side of an arrow is assigned a different axis, shared across conditions.
fn build_axes(
    mut commands: Commands,
    mut query: Query<(&Transform, &ArrowTag, &Path)>,
    mut aes_query: Query<
        (&Distribution<f32>, &Aesthetics, &mut GeomHist),
        (With<Gy>, Without<PopUp>),
    >,
) {
    let mut axes: HashMap<String, HashMap<Side, (Xaxis, Transform)>> = HashMap::new();
    let mut means: HashMap<Side, Vec<f32>> = HashMap::new();
    // first gather all x-limits for different conditions and the arrow and side
    for (dist, aes, mut geom) in aes_query.iter_mut() {
        if geom.in_axis {
            continue;
        }
        means.entry(geom.side.clone()).or_default().push(
            dist.0
                .iter()
                .map(|cloud| cloud.iter().sum::<f32>() / cloud.len() as f32)
                .sum::<f32>()
                / dist.0.len() as f32,
        );
        let xlimits = (
            min_f32(&dist.0.iter().map(|x| min_f32(x)).collect::<Vec<f32>>()),
            max_f32(&dist.0.iter().map(|x| max_f32(x)).collect::<Vec<f32>>()),
        );
        for (trans, arrow, path) in query.iter_mut() {
            if aes.identifiers.iter().any(|r| r == &arrow.id) {
                let size = path_to_vec(path).length();
                let (rotation_90, away) = match geom.side {
                    Side::Right => (-Vec2::Y.angle_between(arrow.direction.perp()), -30.),
                    Side::Left => (-Vec2::NEG_Y.angle_between(arrow.direction.perp()), 30.),
                    _ => {
                        warn!("Tried to plot Up direction for non-popup '{}'", arrow.id);
                        continue;
                    }
                };
                let transform: Transform = if let Some(Some(ser_transform)) =
                    arrow.hists.as_ref().map(|x| x.get(&geom.side))
                {
                    // there were saved histogram positions
                    ser_transform.clone().into()
                } else {
                    // histogram perpendicular to the direction of the arrow
                    // the arrow direction is decided by a fallible heuristic!
                    let mut transform =
                        Transform::from_xyz(trans.translation.x, trans.translation.y, 0.5)
                            .with_rotation(Quat::from_rotation_z(rotation_90));
                    transform.translation.x += arrow.direction.perp().x * away;
                    transform.translation.y += arrow.direction.perp().y * away;
                    transform
                };
                let axis_entry = axes
                    .entry(arrow.id.clone())
                    .or_default()
                    .entry(geom.side.clone())
                    .or_insert((
                        Xaxis {
                            id: arrow.id.clone(),
                            arrow_size: size,
                            xlimits,
                            side: geom.side.clone(),
                            plot: geom.plot.clone(),
                            node_id: arrow.node_id,
                            conditions: Vec::new(),
                        },
                        transform,
                    ));
                axis_entry.0.xlimits = (
                    f32::min(axis_entry.0.xlimits.0, xlimits.0),
                    f32::max(axis_entry.0.xlimits.1, xlimits.1),
                );

                if let Some(cond) = aes.condition.as_ref() {
                    axis_entry.0.conditions.push(cond.clone());
                }
                geom.in_axis = true;
            }
        }
    }
    for (_, _, mut geom) in aes_query.iter_mut() {
        if let Some(side_means) = means.get(&geom.side) {
            geom.mean = Some(side_means.iter().sum::<f32>() / side_means.len() as f32);
        }
    }

    for (axis, trans) in axes.into_values().flat_map(|side| side.into_values()) {
        let size = axis.arrow_size;
        commands.spawn((axis, Drag::default(), plot_line(size, trans)));
    }
}

/// Build axis.
fn build_point_axes(
    mut commands: Commands,
    mut query: Query<(&Transform, &ArrowTag, &Path)>,
    mut aes_query: Query<
        (&Aesthetics, &mut GeomHist),
        (With<Gy>, Without<PopUp>, With<Point<f32>>),
    >,
) {
    let mut axes: HashMap<String, HashMap<Side, (Xaxis, Transform)>> = HashMap::new();
    // first gather all x-limits for different conditions and the arrow and side
    for (aes, mut geom) in aes_query.iter_mut() {
        if geom.in_axis {
            continue;
        }
        for (trans, arrow, path) in query.iter_mut() {
            if aes.identifiers.iter().any(|r| r == &arrow.id) {
                let size = path_to_vec(path).length();
                let (rotation_90, away) = match geom.side {
                    Side::Right => (-Vec2::Y.angle_between(arrow.direction.perp()), -30.),
                    Side::Left => (-Vec2::NEG_Y.angle_between(arrow.direction.perp()), 30.),
                    _ => {
                        warn!("Tried to plot Up direction for non-popup '{}'", arrow.id);
                        continue;
                    }
                };
                let transform: Transform = if let Some(Some(ser_transform)) =
                    arrow.hists.as_ref().map(|x| x.get(&geom.side))
                {
                    // there were saved histogram positions
                    ser_transform.clone().into()
                } else {
                    // histogram perpendicular to the direction of the arrow
                    // the arrow direction is decided by a fallible heuristic!
                    let mut transform =
                        Transform::from_xyz(trans.translation.x, trans.translation.y, 0.5)
                            .with_rotation(Quat::from_rotation_z(rotation_90));
                    transform.translation.x += arrow.direction.perp().x * away;
                    transform.translation.y += arrow.direction.perp().y * away;
                    transform
                };
                let axis_entry = axes
                    .entry(arrow.id.clone())
                    .or_default()
                    .entry(geom.side.clone())
                    .or_insert((
                        Xaxis {
                            id: arrow.id.clone(),
                            arrow_size: size,
                            xlimits: (0., 0.),
                            side: geom.side.clone(),
                            plot: geom.plot.clone(),
                            node_id: arrow.node_id,
                            conditions: Vec::new(),
                        },
                        transform,
                    ));
                if let Some(cond) = aes.condition.as_ref() {
                    axis_entry.0.conditions.push(cond.clone());
                }
                geom.in_axis = true;
            }
        }
    }

    for (mut axis, trans) in axes.into_values().flat_map(|side| side.into_values()) {
        // conditions are sorted everywhere to be consistent across dropdowns, etc
        axis.conditions.sort();
        commands.spawn((
            axis,
            Drag::default(),
            trans,
            Unscale {},
            VisibilityBundle::default(),
        ));
    }
}

fn build_hover_axes(
    mut query: Query<&mut Hover>,
    mut aes_query: Query<(&Distribution<f32>, &Aesthetics, &mut GeomHist), (With<Gy>, With<PopUp>)>,
) {
    let mut axes: HashMap<u64, (f32, f32)> = HashMap::new();
    // first gather all x-limits for different conditions and the arrow and side
    for (dist, aes, mut geom) in aes_query.iter_mut() {
        if geom.in_axis {
            continue;
        }
        for hover in query.iter() {
            if hover.xlimits.is_some() {
                continue;
            }
            if let Some(index) = aes.identifiers.iter().position(|r| r == &hover.id) {
                let this_dist = match dist.0.get(index) {
                    Some(d) => d,
                    None => continue,
                };
                let xlimits = (min_f32(this_dist), max_f32(this_dist));
                let axis_entry = axes.entry(hover.node_id).or_insert(xlimits);
                *axis_entry = (
                    f32::min(axis_entry.0, xlimits.0),
                    f32::max(axis_entry.1, xlimits.1),
                );
                geom.in_axis = true;
            }
        }
    }

    for (node_id, xlimits) in axes {
        for mut hover in query.iter_mut().filter(|h| h.node_id == node_id) {
            hover.xlimits = Some(xlimits)
        }
    }
}

/// Plot histogram as numerical variable next to arrows.
fn plot_side_hist(
    mut commands: Commands,
    mut aes_query: Query<
        (&Distribution<f32>, &Aesthetics, &mut GeomHist, &AesFilter),
        (With<Gy>, Without<PopUp>),
    >,
    query: Query<(&Transform, &Xaxis)>,
) {
    'outer: for (dist, aes, mut geom, is_met) in aes_query.iter_mut() {
        if geom.rendered {
            continue;
        }
        for (trans, axis) in query.iter() {
            if let Some(index) = aes
                .identifiers
                .iter()
                .position(|r| (r == &axis.id) & (geom.side == axis.side))
            {
                let this_dist = match dist.0.get(index) {
                    Some(d) => d,
                    None => continue,
                };
                let line = match geom.plot {
                    HistPlot::Hist => plot_hist(this_dist, 160, axis.arrow_size, axis.xlimits),
                    HistPlot::Kde => plot_kde(this_dist, 100, axis.arrow_size, axis.xlimits),
                    HistPlot::BoxPoint => {
                        warn!("Tried to plot a BoxPoint from a Distributions. Not Implemented! Consider using a Point as input");
                        None
                    }
                };
                let Some(line) = line else { continue 'outer };
                let hex = match geom.side {
                    // the color is updated by another system given the settings
                    Side::Right => "7dce9688",
                    Side::Left => "DA968788",
                    _ => {
                        warn!("Tried to plot Up direction for non-popup '{}'", axis.id);
                        continue;
                    }
                };

                commands.spawn((
                    ShapeBundle {
                        path: GeometryBuilder::build_as(&line),
                        transform: *trans,
                        ..default()
                    },
                    Fill::color(Color::hex(hex).unwrap()),
                    VisCondition {
                        condition: aes.condition.clone(),
                    },
                    HistTag {
                        side: geom.side.clone(),
                        node_id: axis.node_id,
                        follow_scale: true,
                    },
                    (*is_met).clone(),
                ));
            }
            geom.rendered = true;
        }
    }
}

fn plot_side_box(
    mut commands: Commands,
    ui_state: Res<UiState>,
    mut aes_query: Query<
        (&Point<f32>, &Aesthetics, &mut GeomHist, &AesFilter),
        (With<Gy>, Without<PopUp>),
    >,
    mut query: Query<(&mut Transform, &Xaxis), With<Unscale>>,
) {
    for (colors, aes, mut geom, is_box) in aes_query.iter_mut() {
        if geom.rendered {
            continue;
        }
        let min_val = min_f32(&colors.0);
        let max_val = max_f32(&colors.0);
        let grad = build_grad(
            ui_state.zero_white,
            min_val,
            max_val,
            &ui_state.min_reaction_color,
            &ui_state.max_reaction_color,
        );

        for (mut trans, axis) in query.iter_mut() {
            if let Some(index) = aes
                .identifiers
                .iter()
                .position(|r| (r == &axis.id) & (geom.side == axis.side))
            {
                match geom.plot {
                    HistPlot::Hist | HistPlot::Kde => {
                        warn!(
                            "Tried to plot a distribution from one point. Coercing to a Box Point!"
                        );
                    }
                    _ => (),
                };
                let color = from_grad_clamped(&grad, colors.0[index], min_val, max_val);

                trans.translation.z += 10.;
                let shape = if f32::abs(colors.0[index]) > 1e-7 {
                    let line_box = plot_box_point(
                        axis.conditions.len(),
                        axis.conditions
                            .iter()
                            .position(|x| x == aes.condition.as_ref().unwrap_or(&String::from("")))
                            .unwrap_or(0),
                    );
                    (
                        ShapeBundle {
                            path: GeometryBuilder::build_as(&line_box),
                            transform: trans.with_scale(Vec3::new(1., 1., 1.)),
                            ..default()
                        },
                        Fill::color(color),
                        Stroke::new(Color::BLACK, 2.),
                    )
                } else {
                    let circle_center = if axis.conditions.is_empty() {
                        0.
                    } else {
                        let center = axis
                            .conditions
                            .iter()
                            .position(|x| x == aes.condition.as_ref().unwrap_or(&String::from("")))
                            .unwrap_or(0) as f32
                            * 40.0
                            * 1.2;
                        center - axis.conditions.len() as f32 * 40.0 * 1.2 / 2.
                    };
                    let shape = shapes::Circle {
                        radius: 10.,
                        center: Vec2::new(circle_center, 20.),
                    };
                    (
                        ShapeBundle {
                            path: GeometryBuilder::build_as(&shape),
                            transform: trans.with_scale(Vec3::new(1., 1., 1.)),
                            ..default()
                        },
                        Fill::color(color),
                        Stroke::new(Color::BLACK, 2.),
                    )
                };
                commands.spawn((
                    shape,
                    VisCondition {
                        condition: aes.condition.clone(),
                    },
                    HistTag {
                        side: geom.side.clone(),
                        node_id: axis.node_id,
                        follow_scale: false,
                    },
                    ColorListener {
                        value: colors.0[index],
                        min_val,
                        max_val,
                    },
                    Unscale {},
                    (*is_box).clone(),
                ));
            }
            geom.rendered = true;
        }
    }
}

/// Plot hovered histograms of both metabolites and reactions.
fn plot_hover_hist(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut query: Query<(&Transform, &Hover)>,
    mut aes_query: Query<
        (&Distribution<f32>, &Aesthetics, &mut GeomHist, &AesFilter),
        (With<Gy>, With<PopUp>),
    >,
) {
    'outer: for (dist, aes, mut geom, is_met) in aes_query.iter_mut() {
        if geom.rendered {
            continue;
        }
        let font = asset_server.load("fonts/FiraSans-Bold.ttf");
        for (trans, hover) in query.iter_mut() {
            if hover.xlimits.is_none() {
                continue;
            }
            if let Some(index) = aes.identifiers.iter().position(|r| r == &hover.id) {
                let this_dist = match dist.0.get(index) {
                    Some(d) => d,
                    None => continue,
                };
                let xlimits = hover.xlimits.as_ref().unwrap();
                let line = match geom.plot {
                    HistPlot::Hist => plot_hist(this_dist, 55, 600., *xlimits),
                    HistPlot::Kde => plot_kde(this_dist, 80, 600., *xlimits),
                    HistPlot::BoxPoint => {
                        warn!("Tried to plot a BoxPoint from a Distributions. Not Implemented! Consider using a Point as input");
                        None
                    }
                };
                let Some(line) = line else { continue 'outer };
                let transform = Transform::from_xyz(
                    trans.translation.x + 150.,
                    trans.translation.y + 150.,
                    10.,
                );
                let geometry = ShapeBundle {
                    path: GeometryBuilder::build_as(&line),
                    transform,
                    visibility: Visibility::Hidden,
                    ..default()
                };
                let fill = Fill::color(Color::hex("ffb73388").unwrap());
                let scales = plot_scales(this_dist, 600., font.clone(), 12.);
                commands
                    .spawn((
                        HistTag {
                            side: geom.side.clone(),
                            node_id: hover.node_id,
                            follow_scale: false,
                        },
                        VisCondition {
                            condition: aes.condition.clone(),
                        },
                    ))
                    .insert((geometry, fill))
                    .with_children(|p| {
                        p.spawn(SpriteBundle {
                            texture: asset_server.load("hover.png"),
                            transform: Transform::from_xyz(0., 0., -0.4),
                            ..default()
                        });
                    })
                    .with_children(|parent| {
                        parent.spawn(scales.x_0);
                    })
                    .with_children(|parent| {
                        parent.spawn(scales.x_n);
                    })
                    .with_children(|parent| {
                        parent.spawn(scales.y);
                    })
                    .insert((AnyTag { id: hover.node_id }, (*is_met).clone()));
            }
            geom.rendered = true;
        }
    }
}

/// Normalize the height of histograms to be comparable with each other.
/// It treats the two sides independently.
fn normalize_histogram_height(
    mut ui_state: ResMut<UiState>,
    mut query: Query<
        (
            &mut Transform,
            &mut Path,
            &mut Fill,
            &HistTag,
            &VisCondition,
        ),
        Without<Unscale>,
    >,
) {
    for (mut trans, path, mut fill, hist, condition) in query.iter_mut() {
        let height = max_f32(&path.0.iter().map(|ev| ev.to().y).collect::<Vec<f32>>());
        trans.scale.y = match hist.side {
            Side::Left => ui_state.max_left / height,
            Side::Right => ui_state.max_right / height,
            Side::Up => ui_state.max_top / height,
        };
        let ui_condition = ui_state.condition.clone();
        fill.color = {
            let color_ref = match hist.side {
                Side::Left => &mut ui_state.color_left,
                Side::Right => &mut ui_state.color_right,
                Side::Up => &mut ui_state.color_top,
            };
            let color = match condition.condition.as_ref() {
                Some(cond) => or_color(cond, color_ref, true),
                None => or_color(&ui_condition, color_ref, false),
            };
            Color::rgba_linear(color.r(), color.g(), color.b(), color.a())
        }
    }
}

/// Propagate color from Ui to color component.
fn change_color(
    ui_state: Res<UiState>,
    mut query: Query<(&mut Fill, &HistTag, &ColorListener), With<Stroke>>,
) {
    let mut gradients: HashMap<Side, colorgrad::Gradient> = HashMap::new();
    if ui_state.is_changed() {
        for (mut fill, hist, color) in query.iter_mut() {
            let grad = gradients.entry(hist.side.clone()).or_insert(build_grad(
                ui_state.zero_white,
                color.min_val,
                color.max_val,
                &ui_state.min_reaction_color,
                &ui_state.max_reaction_color,
            ));
            fill.color = from_grad_clamped(grad, color.value, color.min_val, color.max_val);
        }
    }
}

/// Unscale up children of scaled histograms.
fn unscale_histogram_children(
    parents: Query<(Entity, &Children), (With<HistTag>, Without<Unscale>)>,
    mut query: Query<&mut Transform>,
) {
    for (parent, children) in parents.iter() {
        let Ok(scale) = query.get_mut(parent).map(|trans| trans.scale.y) else {
            continue;
        };
        for child in children {
            let Ok(mut trans) = query.get_mut(*child) else {
                continue;
            };
            trans.scale.y = 1. / scale;
        }
    }
}

/// Fill conditions menu.
fn fill_conditions(mut ui_state: ResMut<UiState>, aesthetics: Query<&Aesthetics>) {
    let conditions = {
        let mut conditions = aesthetics
            .iter()
            .filter_map(|a| a.condition.clone())
            .unique()
            .collect::<Vec<String>>();
        conditions.sort();
        conditions
    };
    if conditions
        .iter()
        .any(|cond| !ui_state.conditions.contains(cond))
    {
        if !conditions.is_empty() {
            ui_state.conditions = conditions;
            if !ui_state.conditions.contains(&String::from("ALL")) {
                ui_state.conditions.push(String::from("ALL"));
            }
        } else {
            ui_state.conditions = vec![String::from("")];
            ui_state.condition = String::from("");
        }
        if ui_state.condition.is_empty() {
            ui_state.condition = ui_state.conditions[0].clone();
        }
    }
}

/// Hide histograms that are not in the conditions.
pub fn filter_histograms(
    ui_state: Res<UiState>,
    mut query: Query<(&mut Visibility, &VisCondition), Without<AnyTag>>,
) {
    for (mut vis, cond) in query.iter_mut() {
        if let Some(condition) = &cond.condition {
            if (condition != &ui_state.condition) & (ui_state.condition != "ALL") {
                *vis = Visibility::Hidden;
            } else {
                *vis = Visibility::Visible;
            }
        }
    }
}

/// Coordinate the position of histograms with their hovers.
fn follow_the_axes(
    axes: Query<(&Transform, &Xaxis), Changed<Transform>>,
    mut hists: Query<(&mut Transform, &HistTag), (Without<AnyTag>, Without<Xaxis>)>,
) {
    for (axis_trans, axis) in axes.iter() {
        for (mut trans, hist) in hists.iter_mut() {
            if (axis.node_id == hist.node_id) & (hist.side == axis.side) {
                trans.translation = axis_trans.translation;
                trans.rotation = axis_trans.rotation;
                if hist.follow_scale {
                    trans.scale.x = axis_trans.scale.x;
                }
            }
        }
    }
}
