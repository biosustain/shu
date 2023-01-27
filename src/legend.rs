//! Legend generation on demand.

use bevy::{prelude::*, utils::HashMap};

use crate::{
    aesthetics::{Aesthetics, Gcolor, Point},
    funcplot::{linspace, max_f32, min_f32, ScaleBundle},
    geom::{Drag, GeomArrow, HistPlot, Side, Xaxis},
    gui::UiState,
};

pub struct LegendPlugin;

impl Plugin for LegendPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_legend)
            // .add_system(draw_legend_for_axis)
            .add_system(color_legend);
    }
}

#[derive(Component)]
struct Legend;
#[derive(Component)]
struct Xmin;
#[derive(Component)]
struct Xmax;

fn spawn_legend(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let scales = ScaleBundle::new(0., 0., 0., 200., 200., font, 20.);
    let img_handle = asset_server.load("arrow_grad.png");
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Px(400.), Val::Px(50.)),
                padding: UiRect::left(Val::Px(10.0)),
                ..Default::default()
            },
            ..Default::default()
        })
        // legend arrow
        .with_children(|p| {
            p.spawn(NodeBundle {
                style: Style {
                    size: Size::new(Val::Px(320.), Val::Px(50.)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Legend)
            .with_children(|p| {
                p.spawn((
                    TextBundle {
                        text: scales.x_0.text,
                        ..default()
                    },
                    Xmin,
                ));
            })
            .with_children(|p| {
                p.spawn(ImageBundle {
                    style: Style {
                        size: Size::new(Val::Px(200.0), Val::Px(50.0)),
                        ..default()
                    },
                    image: UiImage(img_handle),
                    ..default()
                });
            })
            .with_children(|p| {
                p.spawn((
                    TextBundle {
                        text: scales.x_n.text,
                        ..default()
                    },
                    Xmax,
                ));
            });
        })
        .insert(Drag::default());
}

fn color_legend(
    ui_state: Res<UiState>,
    mut legend_query: Query<(Entity, &Children), With<Legend>>,
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
        for (_parent, children) in &mut legend_query {
            let min_val = min_f32(&colors.0);
            let max_val = max_f32(&colors.0);
            let grad = crate::funcplot::build_grad(
                ui_state.zero_white,
                min_val,
                max_val,
                &ui_state.min_reaction_color,
                &ui_state.max_reaction_color,
            );
            // modify the image inplace
            for child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(*child) {
                    text.sections[0].value = format!("{:.2e}", min_val);
                } else if let Ok(mut text) = text_max_query.get_mut(*child) {
                    text.sections[0].value = format!("{:.2e}", max_val);
                } else if let Ok(img_legend) = img_query.get_mut(*child) {
                    let handle = images.get_handle(&img_legend.0);
                    let image = images.get_mut(&handle).unwrap();
                    let points = linspace(min_val, max_val, 400);
                    let data = image.data.chunks(4).enumerate().flat_map(|(i, pixel)| {
                        let row = (i as f64 / 400.).floor() as f64;
                        let x = i as f64 - 400. * row;
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
