//! This module contains the code for spawning the legend, which is a
//! very verbose flexbox layout.

use bevy::prelude::*;

use crate::{
    funcplot::ScaleBundle,
    geom::{Drag, Side},
};

// parameters for legend sizes
const WIDTH: Val = Val::Px(230.0);
const HEIGHT: Val = Val::Px(240.0);
const HEIGHT_CHILD: Val = Val::Px(40.0);
const HIST_HEIGHT_CHILD: Val = Val::Px(60.0);
const ARROW_BUNDLE_WIDTH: Val = Val::Px(210.0);
const ARROW_WIDTH: Val = Val::Px(120.0);
const ARROW_HEIGHT: Val = Val::Px(22.);
const CIRCLE_BUNDLE_WIDTH: Val = Val::Px(120.0);
const CIRCLE_DIAM: Val = Val::Px(35.0);

#[derive(Component)]
pub struct LegendArrow;
#[derive(Component)]
pub struct LegendCircle;
#[derive(Component)]
pub struct LegendCondition {
    /// Current conditions for change detection.
    pub state: Vec<String>,
}
#[derive(Component)]
pub struct LegendHist;
#[derive(Component)]
pub struct LegendBox;
#[derive(Component)]
pub struct Xmin;
#[derive(Component)]
pub struct Xmax;

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
pub fn spawn_legend(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/Assistant-Regular.ttf");
    let scales_arrow = ScaleBundle::new(
        0.,
        0.,
        0.,
        200.,
        200.,
        font,
        15.,
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
                max_size: Size::new(WIDTH, HEIGHT),
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
            focus_policy: bevy::ui::FocusPolicy::Block,
            z_index: ZIndex::Global(10),
            ..Default::default()
        })
        .insert((Drag::default(), Interaction::default()))
        // box-point legend
        .with_children(|p| {
            // container for both box sides
            p.spawn(NodeBundle {
                style: Style {
                    max_size: Size::new(ARROW_BUNDLE_WIDTH, HIST_HEIGHT_CHILD / 2.0),
                    display: Display::Flex,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceEvenly,
                    ..Default::default()
                },
                focus_policy: bevy::ui::FocusPolicy::Pass,
                ..Default::default()
            })
            // container for left box side with text tags for axis
            .with_children(|p| {
                p.spawn(NodeBundle {
                    style: Style {
                        size: Size::new(ARROW_BUNDLE_WIDTH / 2.3, HIST_HEIGHT_CHILD / 2.0),
                        display: Display::None,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        ..Default::default()
                    },
                    focus_policy: bevy::ui::FocusPolicy::Pass,
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
                            size: Size::new(CIRCLE_DIAM * 0.5, CIRCLE_DIAM * 0.5),
                            ..default()
                        },
                        focus_policy: bevy::ui::FocusPolicy::Pass,
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
                        size: Size::new(ARROW_BUNDLE_WIDTH / 2.3, HIST_HEIGHT_CHILD / 2.0),
                        display: Display::None,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        ..Default::default()
                    },
                    focus_policy: bevy::ui::FocusPolicy::Pass,
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
                            size: Size::new(CIRCLE_DIAM * 0.5, CIRCLE_DIAM * 0.5),
                            ..default()
                        },
                        focus_policy: bevy::ui::FocusPolicy::Pass,
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
        })
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
                focus_policy: bevy::ui::FocusPolicy::Pass,
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
                    focus_policy: bevy::ui::FocusPolicy::Pass,
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
                focus_policy: bevy::ui::FocusPolicy::Pass,
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
                        size: Size::new(CIRCLE_DIAM, CIRCLE_DIAM * 0.8),
                        ..default()
                    },
                    focus_policy: bevy::ui::FocusPolicy::Pass,
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
                    min_size: Size::new(ARROW_BUNDLE_WIDTH, Val::Px(0.0)),
                    max_size: Size::new(ARROW_BUNDLE_WIDTH, HIST_HEIGHT_CHILD * 2.0),
                    display: Display::Flex,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                focus_policy: bevy::ui::FocusPolicy::Pass,
                ..Default::default()
            })
            // condition container
            .with_children(|p| {
                p.spawn((
                    NodeBundle {
                        style: Style {
                            size: Size::new(ARROW_BUNDLE_WIDTH / 6.0, HIST_HEIGHT_CHILD),
                            display: Display::None,
                            margin: UiRect::right(Val::Px(5.0)),
                            flex_direction: FlexDirection::Column,
                            flex_shrink: 1.,
                            align_items: AlignItems::FlexEnd,
                            justify_content: JustifyContent::SpaceAround,
                            ..Default::default()
                        },
                        focus_policy: bevy::ui::FocusPolicy::Pass,
                        ..default()
                    },
                    LegendCondition { state: Vec::new() },
                ));
            })
            // container for left histogram side with text tags for axis
            .with_children(|p| {
                p.spawn(NodeBundle {
                    style: Style {
                        max_size: Size::new(ARROW_BUNDLE_WIDTH / 3.0, HIST_HEIGHT_CHILD * 2.0),
                        display: Display::None,
                        align_items: AlignItems::FlexEnd,
                        flex_direction: FlexDirection::Column,
                        margin: UiRect::right(Val::Px(5.0)),
                        flex_shrink: 3.,
                        justify_content: JustifyContent::Center,
                        ..Default::default()
                    },
                    focus_policy: bevy::ui::FocusPolicy::Pass,
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
                            size: Size::new(HIST_HEIGHT_CHILD * 0.6, HIST_HEIGHT_CHILD),
                            ..default()
                        },
                        focus_policy: bevy::ui::FocusPolicy::Pass,
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
                        max_size: Size::new(ARROW_BUNDLE_WIDTH / 3.0, HIST_HEIGHT_CHILD * 2.),
                        display: Display::None,
                        align_items: AlignItems::FlexStart,
                        margin: UiRect::left(Val::Px(5.0)),
                        flex_shrink: 1.,
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        ..Default::default()
                    },
                    focus_policy: bevy::ui::FocusPolicy::Pass,
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
                            size: Size::new(HIST_HEIGHT_CHILD * 0.6, HIST_HEIGHT_CHILD),
                            ..default()
                        },
                        image: UiImage(hist_right_handle),
                        focus_policy: bevy::ui::FocusPolicy::Pass,
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
        });
}
