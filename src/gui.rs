//! Gui (windows and panels) to upload data and hover.

use crate::data::{MetaboliteData, ReactionData, ReactionState};
use bevy::prelude::*;

pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(ui_example).add_system(file_drop);
    }
}

fn ui_example(mut commands: Commands, asset_server: Res<AssetServer>) {
    // root node
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // left vertical fill (border)
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Px(170.0), Val::Percent(30.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    background_color: Color::rgb(0.15, 0.15, 0.15).into(),
                    ..default()
                })
                .with_children(|parent| {
                    // left vertical fill (content)
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                flex_wrap: FlexWrap::Wrap,
                                flex_direction: FlexDirection::Column,
                                ..default()
                            },
                            background_color: Color::rgb(0.95, 0.95, 0.95).into(),
                            ..default()
                        })
                        .insert(MyDropTarget)
                        .with_children(|parent| {
                            // text
                            parent.spawn(
                                TextBundle::from_section(
                                    "Drop data!",
                                    TextStyle {
                                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                        font_size: 30.0,
                                        color: Color::rgb(0.15, 0.15, 0.15).into(),
                                    },
                                )
                                .with_style(Style {
                                    margin: UiRect::all(Val::Px(5.0)),
                                    ..default()
                                }),
                            );
                        })
                        .with_children(|parent| {
                            parent.spawn(
                                TextBundle::from_section(
                                    "Reaction data should end\nwith '.reaction.json",
                                    TextStyle {
                                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                        font_size: 15.0,
                                        color: Color::rgb(0.15, 0.15, 0.15).into(),
                                    },
                                )
                                .with_style(Style {
                                    margin: UiRect::all(Val::Px(5.0)),
                                    max_size: Size::new(Val::Percent(50.0), Val::Percent(100.0)),
                                    flex_wrap: FlexWrap::Wrap,
                                    ..default()
                                }),
                            );
                        });
                });
        });
}
#[derive(Component)]
struct MyDropTarget;

fn file_drop(
    mut dnd_evr: EventReader<FileDragAndDrop>,
    asset_server: Res<AssetServer>,
    mut reaction_resource: ResMut<ReactionState>,
    query_ui_droptarget: Query<&Interaction, With<MyDropTarget>>,
) {
    for ev in dnd_evr.iter() {
        if let FileDragAndDrop::DroppedFile { id, path_buf } = ev {
            println!("Dropped file with path: {:?}", path_buf);

            if id.is_primary() {
                // it was dropped over the main window
            }

            // it was dropped over our UI element
            // (our UI element is being hovered over)

            if path_buf.to_str().unwrap().ends_with("reaction.json") {
                let reaction_handle: Handle<ReactionData> =
                    asset_server.load(path_buf.to_str().unwrap());
                reaction_resource.reaction_data = Some(reaction_handle);
                reaction_resource.reac_loaded = false;
                info! {"Reactions dropped!"};
            } else {
                let metabolite_handle: Handle<MetaboliteData> =
                    asset_server.load(path_buf.to_str().unwrap());
                reaction_resource.metabolite_data = Some(metabolite_handle);
                reaction_resource.met_loaded = false;
                info! {"Metabolites dropped!"};
            }
        }
    }
}
