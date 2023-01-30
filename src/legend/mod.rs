//! Legend generation on demand.

use bevy::prelude::*;

use crate::{
    aesthetics::{Aesthetics, Gcolor, Gy, Point, Unscale},
    funcplot::{linspace, max_f32, min_f32},
    geom::{GeomArrow, GeomHist, GeomMetabolite, PopUp, Side, Xaxis},
    gui::UiState,
};

mod setup;
use setup::{spawn_legend, LegendArrow, LegendBox, LegendCircle, LegendHist, Xmin};

/// Procedural legend generation.
pub struct LegendPlugin;

impl Plugin for LegendPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_legend)
            .add_system(color_legend_arrow)
            .add_system(color_legend_circle)
            .add_system(color_legend_histograms)
            .add_system(color_legend_box);
    }
}

/// If a [`GeomArrow`] with color is added, and arrow is displayed showcasing the color scale with a gradient.
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
                    let handle = images.get_handle(&img_legend.0);
                    let image = images.get_mut(&handle).unwrap();

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

/// If [`GeomMetabolite`] with color is added, and arrow is displayed showcasing the color scale with a gradient.
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
                    let handle = images.get_handle(&img_legend.0);
                    let image = images.get_mut(&handle).unwrap();

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

/// When a new Right or Left histogram `Xaxis` is spawned, add a legend corresponding to that axis.
fn color_legend_histograms(
    ui_state: Res<UiState>,
    mut legend_query: Query<(Entity, &mut Style, &Side, &Children), With<LegendHist>>,
    // Unscale means that it is not a histogram
    axis_query: Query<&Xaxis, Without<Unscale>>,
    mut img_query: Query<&mut BackgroundColor>,
    mut text_query: Query<&mut Text, With<Xmin>>,
    mut text_max_query: Query<&mut Text, Without<Xmin>>,
) {
    let mut left: Option<((f32, f32), &Side)> = None;
    let mut right: Option<((f32, f32), &Side)> = None;
    // gather all axis limits
    for axis in axis_query.iter() {
        if left.is_some() & right.is_some() {
            break;
        }
        match axis.side {
            Side::Left if left.is_none() => left = Some((axis.xlimits, &axis.side)),
            Side::Right if right.is_none() => right = Some((axis.xlimits, &axis.side)),
            _ => continue,
        }
    }
    // if an axis matches the legend in side, show the legend with bounds and color
    for axis in [left, right].iter().filter_map(|o| o.as_ref()) {
        for (_parent, mut style, side, children) in &mut legend_query {
            for child in children.iter() {
                if axis.1 == side {
                    if let Ok(mut text) = text_query.get_mut(*child) {
                        text.sections[0].value = format!("{:.2e}", axis.0 .0);
                    } else if let Ok(mut text) = text_max_query.get_mut(*child) {
                        text.sections[0].value = format!("{:.2e}", axis.0 .1);
                    } else if let Ok(mut color) = img_query.get_mut(*child) {
                        style.display = Display::Flex;
                        color.0 = match side {
                            Side::Left => {
                                let color = ui_state.color_left;
                                Color::rgba_linear(color.r(), color.g(), color.b(), color.a())
                            }
                            Side::Right => {
                                let color = ui_state.color_right;
                                Color::rgba_linear(color.r(), color.g(), color.b(), color.a())
                            }
                            _ => panic!("unexpected side"),
                        };
                    }
                }
            }
        }
    }
}

/// If a [`GeomArrow`] with color is added, and arrow is displayed showcasing the color scale with a gradient.
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
                if condition != &ui_state.condition {
                    continue;
                }
            }
            if geom_hist.side != *side {
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
                    let handle = images.get_handle(&img_legend.0);
                    let image = images.get_mut(&handle).unwrap();

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
