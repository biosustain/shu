//! Legend generation on demand.

use bevy::{prelude::*, utils::HashMap};

use crate::{
    aesthetics::{Aesthetics, Gcolor, Point},
    funcplot::{linspace, max_f32, min_f32, ScaleBundle},
    geom::{GeomArrow, HistPlot, Side, Xaxis, GeomMetabolite, Drag},
    gui::UiState,
};

pub struct LegendPlugin;

impl Plugin for LegendPlugin {
 
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_legend)
            // .add_system(draw_legend_for_axis)
            .add_system(color_legend_arrow)
            .add_system(color_legend_circle);
    }
}

#[derive(Component)]
struct LegendArrow;
#[derive(Component)]
struct LegendCircle;
#[derive(Component)]
struct Xmin;
#[derive(Component)]
struct Xmax;

fn spawn_legend(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load("fonts/Assistant-Regular.ttf");
    let scales_arrow = ScaleBundle::new(0., 0., 0., 200., 200., font.clone(), 20., Color::hex("504d50").unwrap());
    let scales_mets = ScaleBundle::new(0., 0., 0., 200., 200., font, 20., Color::hex("504d50").unwrap());
    let arrow_handle = asset_server.load("arrow_grad.png");
    let met_handle = asset_server.load("met_grad.png");
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Px(300.), Val::Px(100.)),
                flex_direction: FlexDirection::Column,
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
                    size: Size::new(Val::Px(280.), Val::Px(50.)),
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
                        size: Size::new(Val::Px(150.0), Val::Px(40.0)),
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
                    size: Size::new(Val::Px(160.), Val::Px(50.)),
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
                        size: Size::new(Val::Px(35.0), Val::Px(35.0)),
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
        });
}

fn color_legend_arrow(
    ui_state: Res<UiState>,
    mut legend_query: Query<(Entity, &mut Style, &Children), With<LegendArrow>>,
    mut img_query: Query<&UiImage>,
    mut text_query: Query<&mut Text, With<Xmin>>,
    mut text_max_query: Query<&mut Text, Without<Xmin>>,
    point_query: Query<(&Point<f32>, &Aesthetics), (With<Gcolor>, With<GeomArrow>)>,
    mut images: ResMut<Assets<Image>>,
) {
    for (colors, aes) in point_query.iter() {
        if let Some(condition) = &aes.condition {
            if condition != &ui_state.condition {
                continue;
            }
        }
        for (_parent, mut style, children) in &mut legend_query {
            style.display = Display::Flex;
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
    }
}

fn color_legend_circle(
    ui_state: Res<UiState>,
    mut legend_query: Query<(Entity, &mut Style, &Children), With<LegendCircle>>,
    mut img_query: Query<&UiImage>,
    mut text_query: Query<&mut Text, With<Xmin>>,
    mut text_max_query: Query<&mut Text, Without<Xmin>>,
    point_query: Query<(&Point<f32>, &Aesthetics), (With<Gcolor>, With<GeomMetabolite>)>,
    mut images: ResMut<Assets<Image>>,
) {
    for (colors, aes) in point_query.iter() {
        if let Some(condition) = &aes.condition {
            if condition != &ui_state.condition {
                continue;
            }
        }
        for (_parent, mut style, children) in &mut legend_query {
            style.display = Display::Flex;
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
    }
}
