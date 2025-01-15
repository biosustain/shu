//! This module contains the code for spawning the legend, which is a
//! very verbose flexbox layout.

use bevy::prelude::*;

use crate::{
    funcplot::ScaleBundle,
    geom::{Drag, Side},
    gui::{move_ui_on_drag, recolor_background_on},
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

fn build_image(
    img_handle: Handle<Image>,
    width: Val,
    height: Val,
) -> (ImageNode, Node, bevy::ui::FocusPolicy) {
    (
        ImageNode::new(img_handle),
        Node {
            width,
            height,
            ..default()
        },
        bevy::ui::FocusPolicy::Pass,
    )
}

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
    let scales_arrow = ScaleBundle::<Text>::new(
        0.,
        0.,
        0.,
        200.,
        200.,
        font,
        15.,
        Color::Srgba(bevy::color::Srgba::hex("504d50").unwrap()),
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
        .spawn((
            Node {
                max_width: WIDTH,
                max_height: HEIGHT,
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                left: Val::Px(10.),
                bottom: Val::Px(10.),
                ..Default::default()
            },
            bevy::ui::FocusPolicy::Block,
            GlobalZIndex(10),
        ))
        .observe(move_ui_on_drag)
        .observe(recolor_background_on::<Pointer<Over>>(Color::srgba(
            0.9, 0.9, 0.9, 0.2,
        )))
        .observe(recolor_background_on::<Pointer<Out>>(Color::srgba(
            1.0, 1.0, 1.0, 0.0,
        )))
        .observe(move_ui_on_drag)
        .insert((Drag::default(), Interaction::default()))
        // box-point legend
        .with_children(|p| {
            // container for both box sides
            p.spawn((
                Node {
                    max_width: ARROW_BUNDLE_WIDTH,
                    max_height: HIST_HEIGHT_CHILD / 2.0,
                    display: Display::Flex,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceEvenly,
                    ..Default::default()
                },
                bevy::ui::FocusPolicy::Pass,
            ))
            // container for left box side with text tags for axis
            .with_children(|p| {
                p.spawn((
                    Node {
                        width: ARROW_BUNDLE_WIDTH,
                        height: HIST_HEIGHT_CHILD / 2.0,
                        display: Display::None,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        ..Default::default()
                    },
                    bevy::ui::FocusPolicy::Pass,
                ))
                .insert(LegendBox)
                .insert(Side::Left)
                // left box side
                .with_children(|p| {
                    // TODO: check this works as expected
                    p.spawn((
                        scales_right_box.x_0.0,
                        scales_right_box.x_0.1,
                        scales_right_box.x_0.2,
                        Xmin,
                    ));
                })
                .with_children(|p| {
                    p.spawn(build_image(
                        box_handle.clone(),
                        CIRCLE_DIAM * 0.5,
                        CIRCLE_DIAM * 0.5,
                    ));
                })
                .with_children(|p| {
                    p.spawn((
                        scales_right_box.x_n.0,
                        scales_right_box.x_n.1,
                        scales_right_box.x_n.2,
                        Xmax,
                    ));
                });
            })
            // container for right box side with text tags for axis
            .with_children(|p| {
                p.spawn((
                    Node {
                        width: ARROW_BUNDLE_WIDTH / 2.3,
                        height: HIST_HEIGHT_CHILD / 2.0,
                        display: Display::None,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        ..Default::default()
                    },
                    bevy::ui::FocusPolicy::Pass,
                ))
                .insert(LegendBox)
                .insert(Side::Right)
                // right box side
                .with_children(|p| {
                    p.spawn((
                        scales_left_box.x_0.0,
                        scales_left_box.x_0.1,
                        scales_left_box.x_0.2,
                        Xmin,
                    ));
                })
                .with_children(|p| {
                    p.spawn(build_image(
                        box_handle.clone(),
                        CIRCLE_DIAM * 0.5,
                        CIRCLE_DIAM * 0.5,
                    ));
                })
                .with_children(|p| {
                    p.spawn((
                        scales_left_box.x_n.0,
                        scales_left_box.x_n.1,
                        scales_left_box.x_n.2,
                        Xmax,
                    ));
                });
            });
        })
        // arrow legend
        .with_children(|p| {
            p.spawn((
                Node {
                    display: Display::None,
                    width: ARROW_BUNDLE_WIDTH,
                    height: HEIGHT_CHILD,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    ..Default::default()
                },
                bevy::ui::FocusPolicy::Pass,
            ))
            .insert(LegendArrow)
            .with_children(|p| {
                p.spawn((
                    scales_arrow.x_0.0,
                    scales_arrow.x_0.1,
                    scales_arrow.x_0.2,
                    Xmin,
                ));
            })
            .with_children(|p| {
                p.spawn(build_image(arrow_handle.clone(), ARROW_WIDTH, ARROW_HEIGHT));
            })
            .with_children(|p| {
                p.spawn((
                    scales_arrow.x_n.0,
                    scales_arrow.x_n.1,
                    scales_arrow.x_n.2,
                    Xmax,
                ));
            });
        })
        // metabolite legend
        .with_children(|p| {
            p.spawn((
                Node {
                    width: CIRCLE_BUNDLE_WIDTH,
                    height: HEIGHT_CHILD,
                    display: Display::None,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    ..Default::default()
                },
                bevy::ui::FocusPolicy::Pass,
            ))
            .insert(LegendCircle)
            .with_children(|p| {
                p.spawn((
                    scales_mets.x_0.0,
                    scales_mets.x_0.1,
                    scales_mets.x_0.2,
                    Xmin,
                ));
            })
            .with_children(|p| {
                p.spawn(build_image(
                    met_handle.clone(),
                    CIRCLE_DIAM,
                    CIRCLE_DIAM * 0.8,
                ));
            })
            .with_children(|p| {
                p.spawn((
                    scales_mets.x_n.0,
                    scales_mets.x_n.1,
                    scales_mets.x_n.2,
                    Xmax,
                ));
            });
        })
        // hist legend
        .with_children(|p| {
            // container for both histogram sides
            p.spawn((
                Node {
                    width: ARROW_BUNDLE_WIDTH,
                    min_height: Val::Px(0.0),
                    max_height: HIST_HEIGHT_CHILD * 2.0,
                    display: Display::Flex,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                bevy::ui::FocusPolicy::Pass,
            ))
            // condition container
            .with_children(|p| {
                p.spawn((
                    Node {
                        width: ARROW_BUNDLE_WIDTH / 6.0,
                        height: HIST_HEIGHT_CHILD,
                        display: Display::None,
                        margin: UiRect::right(Val::Px(5.0)),
                        flex_direction: FlexDirection::Column,
                        flex_shrink: 1.,
                        align_items: AlignItems::FlexEnd,
                        justify_content: JustifyContent::SpaceAround,
                        ..Default::default()
                    },
                    bevy::ui::FocusPolicy::Pass,
                    LegendCondition { state: Vec::new() },
                ));
            })
            // container for left histogram side with text tags for axis
            .with_children(|p| {
                p.spawn((
                    Node {
                        max_width: ARROW_BUNDLE_WIDTH / 3.0,
                        max_height: HIST_HEIGHT_CHILD * 2.0,
                        display: Display::None,
                        align_items: AlignItems::FlexEnd,
                        flex_direction: FlexDirection::Column,
                        margin: UiRect::right(Val::Px(5.0)),
                        flex_shrink: 3.,
                        justify_content: JustifyContent::Center,
                        ..Default::default()
                    },
                    bevy::ui::FocusPolicy::Pass,
                ))
                .insert(LegendHist)
                .insert(Side::Left)
                // left histogram side
                .with_children(|p| {
                    p.spawn((
                        scales_left.x_0.0,
                        scales_left.x_0.1,
                        scales_left.x_0.2,
                        Xmin,
                    ));
                })
                .with_children(|p| {
                    p.spawn(build_image(
                        hist_left_handle.clone(),
                        HIST_HEIGHT_CHILD * 0.6,
                        HIST_HEIGHT_CHILD,
                    ));
                })
                .with_children(|p| {
                    p.spawn((
                        scales_left.x_n.0,
                        scales_left.x_n.1,
                        scales_left.x_n.2,
                        Xmax,
                    ));
                });
            })
            // container for right histogram side with text tags for axis
            .with_children(|p| {
                p.spawn((
                    Node {
                        max_width: ARROW_BUNDLE_WIDTH / 3.0,
                        max_height: HIST_HEIGHT_CHILD * 2.,
                        display: Display::None,
                        align_items: AlignItems::FlexStart,
                        margin: UiRect::left(Val::Px(5.0)),
                        flex_shrink: 1.,
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        ..Default::default()
                    },
                    bevy::ui::FocusPolicy::Pass,
                ))
                .insert(LegendHist)
                .insert(Side::Right)
                // right histogram side
                .with_children(|p| {
                    p.spawn((
                        scales_right.x_0.0,
                        scales_right.x_0.1,
                        scales_right.x_0.2,
                        Xmin,
                    ));
                })
                .with_children(|p| {
                    p.spawn(build_image(
                        hist_right_handle.clone(),
                        HIST_HEIGHT_CHILD * 0.6,
                        HIST_HEIGHT_CHILD,
                    ));
                })
                .with_children(|p| {
                    p.spawn((
                        scales_right.x_n.0,
                        scales_right.x_n.1,
                        scales_right.x_n.2,
                        Xmax,
                    ));
                });
            });
        });
}
