//! Procedural legend generation.

use bevy::prelude::*;

use crate::{
    aesthetics::{Aesthetics, Distribution, Gcolor, Gy, Point, Unscale},
    funcplot::{linspace, max_f32, min_f32},
    geom::{GeomArrow, GeomHist, GeomMetabolite, PopUp, Side, Xaxis},
    gui::{or_color, UiState},
};

mod setup;
use setup::{spawn_legend, LegendArrow, LegendBox, LegendCircle};
pub use setup::{LegendCondition, LegendHist, Xmax, Xmin};

/// Procedural legend generation.
pub struct LegendPlugin;

impl Plugin for LegendPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_legend).add_systems(
            Update,
            (
                color_legend_arrow,
                color_legend_circle,
                color_legend_histograms,
                color_legend_box,
                display_conditions,
            ),
        );
    }
}

/// If a [`GeomArrow`] with color is added, and arrow is displayed showcasing the color scale with a gradient.
///
/// The legend is displayed only if there is data with the right aes [`Gcolor`] and geom [`GeomArrow`].
///
/// # Conditions
///
/// * If the data comes with `None` condition, the legend is always displayed.
/// * If the data comes with `Some` condition only the selected condition is displayed.
/// * If "ALL" conditions are selected, the legend is displayed for the last condition,
///   which is the one that is displayed on the map.
fn color_legend_arrow(
    ui_state: Res<UiState>,
    mut legend_query: Query<(Entity, &mut Style, &Children), With<LegendArrow>>,
    mut img_query: Query<&UiImage>,
    mut text_query: Query<&mut Text, With<Xmin>>,
    mut text_max_query: Query<&mut Text, Without<Xmin>>,
    point_query: Query<(&Point<f32>, &Aesthetics), (With<Gcolor>, With<GeomArrow>)>,
    mut images: ResMut<Assets<Image>>,
) {
    for (_parent, mut style, children) in &mut legend_query {
        let mut displayed = Display::None;
        for (colors, aes) in point_query.iter() {
            if let Some(condition) = &aes.condition {
                if condition != &ui_state.condition {
                    if ui_state.condition == "ALL" {
                        // legend should not show if there are no data matching the
                        // geoms and aes even if the condition is "ALL"
                        displayed = Display::Flex;
                    }
                    continue;
                }
            }
            displayed = Display::Flex;
            let min_val = min_f32(&colors.0);
            let max_val = max_f32(&colors.0);
            let grad = crate::funcplot::build_grad(
                ui_state.zero_white,
                min_val,
                max_val,
                &ui_state.min_reaction_color,
                &ui_state.max_reaction_color,
            );
            for child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(*child) {
                    text.sections[0].value = format!("{:.2e}", min_val);
                } else if let Ok(mut text) = text_max_query.get_mut(*child) {
                    text.sections[0].value = format!("{:.2e}", max_val);
                } else if let Ok(img_legend) = img_query.get_mut(*child) {
                    // modify the image inplace
                    let img = images.get_mut(&img_legend.texture).unwrap();

                    let width = img.size().x as f64;
                    let points = linspace(min_val, max_val, width as u32);
                    let data = img.data.chunks(4).enumerate().flat_map(|(i, pixel)| {
                        let row = (i as f64 / width).floor();
                        let x = i as f64 - width * row;
                        if pixel[3] != 0 {
                            let color = grad.at(points[x as usize] as f64).to_rgba8();
                            [color[0], color[1], color[2], color[3]].into_iter()
                        } else {
                            [0, 0, 0, 0].into_iter()
                        }
                    });
                    img.data = data.collect::<Vec<u8>>();
                }
            }
        }
        style.display = displayed;
    }
}

/// If [`GeomMetabolite`] with color is added, and arrow is displayed showcasing the color scale with a gradient.
///
/// The legend is displayed only if there is data with the right aes [`Gcolor`] and geom [`GeomMetabolite`].
///
/// # Conditions
///
/// * If the data comes with `None` condition, the legend is always displayed.
/// * If the data comes with `Some` condition only the selected condition is displayed.
/// * If "ALL" conditions are selected, the legend is displayed for the last condition,
///   which is the one that is displayed on the map.
fn color_legend_circle(
    ui_state: Res<UiState>,
    mut legend_query: Query<(Entity, &mut Style, &Children), With<LegendCircle>>,
    mut img_query: Query<&UiImage>,
    mut text_query: Query<&mut Text, With<Xmin>>,
    mut text_max_query: Query<&mut Text, Without<Xmin>>,
    point_query: Query<(&Point<f32>, &Aesthetics), (With<Gcolor>, With<GeomMetabolite>)>,
    mut images: ResMut<Assets<Image>>,
) {
    for (_parent, mut style, children) in &mut legend_query {
        let mut displayed = Display::None;
        for (colors, aes) in point_query.iter() {
            if let Some(condition) = &aes.condition {
                if condition != &ui_state.condition {
                    if ui_state.condition == "ALL" {
                        displayed = Display::Flex;
                    }
                    continue;
                }
            }
            displayed = Display::Flex;
            let min_val = min_f32(&colors.0);
            let max_val = max_f32(&colors.0);
            let grad = crate::funcplot::build_grad(
                ui_state.zero_white,
                min_val,
                max_val,
                &ui_state.min_metabolite_color,
                &ui_state.max_metabolite_color,
            );
            for child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(*child) {
                    text.sections[0].value = format!("{:.2e}", min_val);
                } else if let Ok(mut text) = text_max_query.get_mut(*child) {
                    text.sections[0].value = format!("{:.2e}", max_val);
                } else if let Ok(img_legend) = img_query.get_mut(*child) {
                    // modify the image inplace
                    let img = images.get_mut(&img_legend.texture).unwrap();

                    let width = img.size().x as f64;
                    let points = linspace(min_val, max_val, width as u32);
                    let data = img.data.chunks(4).enumerate().flat_map(|(i, pixel)| {
                        let row = (i as f64 / width).floor();
                        let x = i as f64 - width * row;
                        if pixel[3] != 0 {
                            let color = grad.at(points[x as usize] as f64).to_rgba8();
                            [color[0], color[1], color[2], color[3]].into_iter()
                        } else {
                            [0, 0, 0, 0].into_iter()
                        }
                    });
                    img.data = data.collect::<Vec<u8>>();
                }
            }
        }
        style.display = displayed;
    }
}

/// When a new Right or Left histogram `Xaxis` is spawned, add a legend corresponding to that axis.
fn color_legend_histograms(
    mut ui_state: ResMut<UiState>,
    mut images: ResMut<Assets<Image>>,
    mut legend_query: Query<(Entity, &mut Style, &Side, &Children), With<LegendHist>>,
    // Unscale means would mean that is not a histogram
    axis_query: Query<&Xaxis, Without<Unscale>>,
    // only queries for collapsing the legend if no hist data is displayed anymore
    hist_query: Query<
        &GeomHist,
        (
            With<Gy>,
            Without<PopUp>,
            With<Aesthetics>,
            With<Distribution<f32>>,
        ),
    >,
    mut img_query: Query<(&UiImage, &mut BackgroundColor)>,
    mut text_query: Query<&mut Text, With<Xmin>>,
    mut text_max_query: Query<&mut Text, Without<Xmin>>,
) {
    if !ui_state.is_changed() {
        // the ui_state always changes on the creation of histograms
        return;
    }
    let mut left: Option<((f32, f32), &Side, bool)> = None;
    let mut right: Option<((f32, f32), &Side, bool)> = None;
    // gather axis limits for each axis if they exist
    for axis in axis_query.iter() {
        if left.is_some() & right.is_some() {
            break;
        }
        let side = match axis.side {
            Side::Left if left.is_none() => &mut left,
            Side::Right if right.is_none() => &mut right,
            _ => continue,
        };
        *side = Some((
            axis.xlimits,
            &axis.side,
            hist_query.iter().any(|hist| hist.side == axis.side),
        ));
    }
    let condition = ui_state.condition.clone();
    // if an axis matches the legend in side, show the legend with bounds and color
    for (xlimits, axis_side, display) in [left, right].iter().filter_map(|o| o.as_ref()) {
        for (_parent, mut style, side, children) in &mut legend_query {
            if !display {
                style.display = Display::None;
                continue;
            }
            for child in children.iter() {
                if axis_side == &side {
                    if let Ok(mut text) = text_query.get_mut(*child) {
                        text.sections[0].value = format!("{:.2e}", xlimits.0);
                    } else if let Ok(mut text) = text_max_query.get_mut(*child) {
                        text.sections[0].value = format!("{:.2e}", xlimits.1);
                    } else {
                        style.display = Display::Flex;
                        if let Ok((img_legend, mut background_color)) = img_query.get_mut(*child) {
                            // modify the image inplace
                            let image = images.get_mut(&img_legend.texture).unwrap();
                            if condition == "ALL" {
                                // show all conditions laminating the legend
                                background_color.0 = Color::rgba_linear(1., 1., 1., 1.);
                                let conditions = ui_state.conditions.clone();
                                let color_ref = match side {
                                    Side::Left => &mut ui_state.color_left,
                                    Side::Right => &mut ui_state.color_right,
                                    _ => panic!("unexpected side"),
                                };

                                let width = image.size().x;
                                let colors: Vec<_> = conditions
                                    .iter()
                                    .filter(|k| (k.as_str() != "") & (k.as_str() != "ALL"))
                                    .map(|k| {
                                        // depending on the order of execution, the colors
                                        // might have not been initialized by the histogram plotter
                                        let cl = or_color(k, color_ref, true);
                                        let c = Color::rgba_linear(cl.r(), cl.g(), cl.b(), cl.a())
                                            .as_rgba();
                                        [
                                            (c.r() * 255.) as u8,
                                            (c.g() * 255.) as u8,
                                            (c.b() * 255.) as u8,
                                            (c.a() * 255.) as u8,
                                        ]
                                    })
                                    .collect();
                                let part = image.size().y / colors.len() as u32;
                                let data =
                                    image.data.chunks(4).enumerate().flat_map(|(i, pixel)| {
                                        let row = i as u32 / width;
                                        let section =
                                            usize::min((row / part) as usize, colors.len() - 1);
                                        if pixel[3] != 0 {
                                            colors[section]
                                        } else {
                                            [0, 0, 0, 0]
                                        }
                                        .into_iter()
                                    });
                                image.data = data.collect::<Vec<u8>>();
                            } else {
                                if background_color.0 == Color::rgba_linear(1., 1., 1., 1.) {
                                    // previous condition was ALL (or never changed)
                                    // reset the image data that was painted with colors
                                    let data = image.data.chunks(4).flat_map(|pixel| {
                                        if pixel[3] != 0 {
                                            [255, 255, 255, 255].into_iter()
                                        } else {
                                            [0, 0, 0, 0].into_iter()
                                        }
                                    });
                                    image.data = data.collect::<Vec<u8>>();
                                }
                                background_color.0 = {
                                    let ref_col = match side {
                                        Side::Left => &mut ui_state.color_left,
                                        Side::Right => &mut ui_state.color_right,
                                        _ => panic!("unexpected side"),
                                    };
                                    let color = or_color(&condition, ref_col, true);
                                    Color::rgba_linear(color.r(), color.g(), color.b(), color.a())
                                };
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Display left and right gradient boxes only if there is such a query like `point_query`,
/// which corresponds to a box-point geom.
///
/// # Conditions
///
/// * If the data comes with `None` condition, the legend is always displayed.
/// * If the data comes with `Some` condition only the selected condition is displayed.
/// * If "ALL" conditions are selected, the legend is displayed for the last condition,
///   which is the one that is displayed on the map.
fn color_legend_box(
    ui_state: Res<UiState>,
    mut legend_query: Query<(Entity, &mut Style, &Side, &Children), With<LegendBox>>,
    mut img_query: Query<&UiImage>,
    mut text_query: Query<&mut Text, With<Xmin>>,
    mut text_max_query: Query<&mut Text, Without<Xmin>>,
    point_query: Query<(&Point<f32>, &Aesthetics, &GeomHist), (With<Gy>, Without<PopUp>)>,
    mut images: ResMut<Assets<Image>>,
) {
    for (_parent, mut style, side, children) in &mut legend_query {
        let mut displayed = Display::None;
        for (colors, aes, geom_hist) in point_query.iter() {
            if let Some(condition) = &aes.condition {
                if (condition != &ui_state.condition) & (ui_state.condition != "ALL") {
                    continue;
                }
            }
            if geom_hist.side != *side {
                displayed = Display::None;
                continue;
            }
            displayed = Display::Flex;
            let min_val = min_f32(&colors.0);
            let max_val = max_f32(&colors.0);
            let grad = crate::funcplot::build_grad(
                ui_state.zero_white,
                min_val,
                max_val,
                &ui_state.min_reaction_color,
                &ui_state.max_reaction_color,
            );
            for child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(*child) {
                    text.sections[0].value = format!("{:.2e}", min_val);
                } else if let Ok(mut text) = text_max_query.get_mut(*child) {
                    text.sections[0].value = format!("{:.2e}", max_val);
                } else if let Ok(img_legend) = img_query.get_mut(*child) {
                    // modify the image inplace
                    let image = images.get_mut(&img_legend.texture).unwrap();

                    let width = image.size().x as f64;
                    let points = linspace(min_val, max_val, width as u32);
                    let data = image.data.chunks(4).enumerate().flat_map(|(i, pixel)| {
                        let row = (i as f64 / width).floor();
                        let x = i as f64 - width * row;
                        if pixel[3] != 0 {
                            let color = grad.at(points[x as usize] as f64).to_rgba8();
                            [color[0], color[1], color[2], color[3]].into_iter()
                        } else {
                            [0, 0, 0, 0].into_iter()
                        }
                    });
                    image.data = data.collect::<Vec<u8>>();
                }
            }
        }
        style.display = displayed;
    }
}

fn display_conditions(
    mut commands: Commands,
    ui_state: Res<UiState>,
    asset_server: Res<AssetServer>,
    mut legend_query: Query<(Entity, &mut Style, &mut LegendCondition)>,
) {
    if !ui_state.is_changed() {
        return;
    }
    if (ui_state.condition != "ALL") || ui_state.conditions.is_empty() {
        for (_, mut style, _) in &mut legend_query {
            style.display = Display::None;
        }
        return;
    }
    let font = asset_server.load("fonts/Assistant-Regular.ttf");
    let conditions = ui_state
        .conditions
        .iter()
        .filter(|k| (k.as_str() != "") & (k.as_str() != "ALL"))
        .cloned()
        .collect::<Vec<_>>();

    for (parent, mut style, mut legend) in &mut legend_query {
        style.display = Display::Flex;
        if legend.state != conditions {
            commands.entity(parent).despawn_descendants();
            legend.state = conditions.clone();
            // commands.entity(parent).remove_children(children);
            conditions.iter().for_each(|text| {
                commands.entity(parent).with_children(|p| {
                    p.spawn(TextBundle {
                        text: Text::from_section(
                            text,
                            TextStyle {
                                font: font.clone(),
                                font_size: 12.,
                                color: Color::hex("504d50").unwrap(),
                            },
                        ),
                        ..Default::default()
                    });
                });
            });
        }
    }
}
