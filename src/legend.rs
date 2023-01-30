//! Legend generation on demand.

use bevy::prelude::*;

use crate::{
    aesthetics::{Aesthetics, Gcolor, Gy, Point, Unscale},
    funcplot::{linspace, max_f32, min_f32, ScaleBundle},
    geom::{Drag, GeomArrow, GeomHist, GeomMetabolite, PopUp, Side, Xaxis},
    gui::UiState,
};

// parameters for legend sizes
const WIDTH: Val = Val::Px(300.0);
const HEIGHT: Val = Val::Px(250.0);
const HEIGHT_CHILD: Val = Val::Px(50.0);
const HIST_HEIGHT_CHILD: Val = Val::Px(80.0);
const ARROW_BUNDLE_WIDTH: Val = Val::Px(280.0);
const ARROW_WIDTH: Val = Val::Px(150.0);
const ARROW_HEIGHT: Val = Val::Px(40.);
const CIRCLE_BUNDLE_WIDTH: Val = Val::Px(160.0);
const CIRCLE_DIAM: Val = Val::Px(35.0);

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

#[derive(Component)]
struct LegendArrow;
#[derive(Component)]
struct LegendCircle;
#[derive(Component)]
struct LegendHist;
#[derive(Component)]
struct LegendBox;
#[derive(Component)]
struct Xmin;
#[derive(Component)]
struct Xmax;

/// Spawn the legend. Nothing is displayed on spawn; only when the user
/// adds data corresponding to a part of the legend, that part is displayed.
///
/// The legend is a Column with 4 row children:
/// - arrow legend with 3 children: Text(min), UiImage(arrow), Text(max).
/// - metabolite legend with 3 children: Text(min), UiImage(circle), Text(max).
/// - histogram legend with 2 column children:
///     - Text(min), UiImage(histogram), Text(max).
///     - Text(min), UiImage(histogram), Text(maximum).
/// - box legend, same as histogram but with Rects instead of images.
fn spawn_legend(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/Assistant-Regular.ttf");
    let scales_arrow = ScaleBundle::new(
        0.,
        0.,
        0.,
        200.,
        200.,
        font,
        20.,
        Color::hex("504d50").unwrap(),
    );
    let scales_mets = scales_arrow.clone();
    let scales_left = scales_arrow.clone();
    let scales_right = scales_arrow.clone();
    let scales_left_box = scales_arrow.clone();
    let scales_right_box = scales_arrow.clone();
    let arrow_handle = asset_server.load("arrow_grad.png");
    let met_handle = asset_server.load("met_grad.png");
    let hist_left_handle = asset_server.load("hist_legend.png");
    let hist_right_handle = asset_server.load("hist_legend_right.png");
    let box_handle = asset_server.load("rect_legend.png");
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(WIDTH, HEIGHT),
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                position: UiRect {
                    left: Val::Px(10.),
                    bottom: Val::Px(10.),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .insert((Drag::default(), Interaction::default()))
        // arrow legend
        .with_children(|p| {
            p.spawn(NodeBundle {
                style: Style {
                    display: Display::None,
                    size: Size::new(ARROW_BUNDLE_WIDTH, HEIGHT_CHILD),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(LegendArrow)
            .with_children(|p| {
                p.spawn((
                    TextBundle {
                        text: scales_arrow.x_0.text,
                        ..default()
                    },
                    Xmin,
                ));
            })
            .with_children(|p| {
                p.spawn(ImageBundle {
                    style: Style {
                        size: Size::new(ARROW_WIDTH, ARROW_HEIGHT),
                        ..default()
                    },
                    image: UiImage(arrow_handle),
                    ..default()
                });
            })
            .with_children(|p| {
                p.spawn((
                    TextBundle {
                        text: scales_arrow.x_n.text,
                        ..default()
                    },
                    Xmax,
                ));
            });
        })
        // metabolite legend
        .with_children(|p| {
            p.spawn(NodeBundle {
                style: Style {
                    size: Size::new(CIRCLE_BUNDLE_WIDTH, HEIGHT_CHILD),
                    display: Display::None,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(LegendCircle)
            .with_children(|p| {
                p.spawn((
                    TextBundle {
                        text: scales_mets.x_0.text,
                        ..default()
                    },
                    Xmin,
                ));
            })
            .with_children(|p| {
                p.spawn(ImageBundle {
                    style: Style {
                        size: Size::new(CIRCLE_DIAM, CIRCLE_DIAM),
                        ..default()
                    },
                    image: UiImage(met_handle),
                    ..default()
                });
            })
            .with_children(|p| {
                p.spawn((
                    TextBundle {
                        text: scales_mets.x_n.text,
                        ..default()
                    },
                    Xmax,
                ));
            });
        })
        // hist legend
        .with_children(|p| {
            // container for both histogram sides
            p.spawn(NodeBundle {
                style: Style {
                    size: Size::new(ARROW_BUNDLE_WIDTH, HIST_HEIGHT_CHILD * 2.0),
                    display: Display::Flex,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    ..Default::default()
                },
                ..Default::default()
            })
            // container for left histogram side with text tags for axis
            .with_children(|p| {
                p.spawn(NodeBundle {
                    style: Style {
                        size: Size::new(ARROW_BUNDLE_WIDTH / 2.2, HIST_HEIGHT_CHILD * 20.),
                        display: Display::None,
                        align_items: AlignItems::FlexEnd,
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(LegendHist)
                .insert(Side::Left)
                // left histogram side
                .with_children(|p| {
                    p.spawn((
                        TextBundle {
                            text: scales_left.x_0.text,
                            ..default()
                        },
                        Xmin,
                    ));
                })
                .with_children(|p| {
                    p.spawn(ImageBundle {
                        style: Style {
                            size: Size::new(HIST_HEIGHT_CHILD, HIST_HEIGHT_CHILD),
                            ..default()
                        },
                        image: UiImage(hist_left_handle),
                        ..default()
                    });
                })
                .with_children(|p| {
                    p.spawn((
                        TextBundle {
                            text: scales_left.x_n.text,
                            ..default()
                        },
                        Xmax,
                    ));
                });
            })
            // container for right histogram side with text tags for axis
            .with_children(|p| {
                p.spawn(NodeBundle {
                    style: Style {
                        size: Size::new(ARROW_BUNDLE_WIDTH / 2.2, HIST_HEIGHT_CHILD * 20.),
                        display: Display::None,
                        align_items: AlignItems::FlexStart,
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(LegendHist)
                .insert(Side::Right)
                // right histogram side
                .with_children(|p| {
                    p.spawn((
                        TextBundle {
                            text: scales_right.x_0.text,
                            ..default()
                        },
                        Xmin,
                    ));
                })
                .with_children(|p| {
                    p.spawn(ImageBundle {
                        style: Style {
                            size: Size::new(HIST_HEIGHT_CHILD, HIST_HEIGHT_CHILD),
                            ..default()
                        },
                        image: UiImage(hist_right_handle),
                        ..default()
                    });
                })
                .with_children(|p| {
                    p.spawn((
                        TextBundle {
                            text: scales_right.x_n.text,
                            ..default()
                        },
                        Xmax,
                    ));
                });
            });
        })
        // box-point legend
        .with_children(|p| {
            // container for both box sides
            p.spawn(NodeBundle {
                style: Style {
                    size: Size::new(ARROW_BUNDLE_WIDTH, HIST_HEIGHT_CHILD),
                    display: Display::Flex,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    ..Default::default()
                },
                ..Default::default()
            })
            // container for left box side with text tags for axis
            .with_children(|p| {
                p.spawn(NodeBundle {
                    style: Style {
                        size: Size::new(ARROW_BUNDLE_WIDTH / 2.2, HIST_HEIGHT_CHILD),
                        display: Display::None,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(LegendBox)
                .insert(Side::Left)
                // left box side
                .with_children(|p| {
                    p.spawn((
                        TextBundle {
                            text: scales_right_box.x_0.text,
                            ..default()
                        },
                        Xmin,
                    ));
                })
                .with_children(|p| {
                    p.spawn(ImageBundle {
                        style: Style {
                            size: Size::new(CIRCLE_DIAM * 0.8, CIRCLE_DIAM * 0.8),
                            ..default()
                        },
                        image: UiImage(box_handle.clone()),
                        ..default()
                    });
                })
                .with_children(|p| {
                    p.spawn((
                        TextBundle {
                            text: scales_right_box.x_n.text,
                            ..default()
                        },
                        Xmax,
                    ));
                });
            })
            // container for right box side with text tags for axis
            .with_children(|p| {
                p.spawn(NodeBundle {
                    style: Style {
                        size: Size::new(ARROW_BUNDLE_WIDTH / 2.2, HIST_HEIGHT_CHILD),
                        display: Display::None,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(LegendBox)
                .insert(Side::Right)
                // right box side
                .with_children(|p| {
                    p.spawn((
                        TextBundle {
                            text: scales_left_box.x_0.text,
                            ..default()
                        },
                        Xmin,
                    ));
                })
                .with_children(|p| {
                    p.spawn(ImageBundle {
                        style: Style {
                            size: Size::new(CIRCLE_DIAM * 0.8, CIRCLE_DIAM * 0.8),
                            ..default()
                        },
                        image: UiImage(box_handle.clone()),
                        ..default()
                    });
                })
                .with_children(|p| {
                    p.spawn((
                        TextBundle {
                            text: scales_left_box.x_n.text,
                            ..default()
                        },
                        Xmax,
                    ));
                });
            });
        });
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
