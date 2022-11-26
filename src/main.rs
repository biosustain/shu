#[allow(type_complexity)]
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::prelude::*;
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_prototype_lyon::prelude::*;

mod aesthetics;
mod data;
mod escher;
mod funcplot;
mod geom;
mod gui;

use escher::{EscherMap, EscherPlugin, MapState};

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(PanCamPlugin::default())
        .add_plugin(ShapePlugin)
        .add_plugin(EscherPlugin)
        .add_plugin(gui::GuiPlugin)
        .add_plugin(data::DataPlugin)
        .add_startup_system(setup_system)
        .add_plugin(aesthetics::AesPlugin)
        .run();
}

fn setup_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let escher_handle: Handle<EscherMap> = asset_server.load("ecoli_core_map.json");
    commands.insert_resource(MapState {
        escher_map: escher_handle,
        loaded: false,
    });
    commands.insert_resource(data::ReactionState {
        reaction_data: None,
        metabolite_data: None,
        reac_loaded: false,
        met_loaded: false,
    });

    commands
        .spawn(Camera2dBundle {
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::Custom(Color::rgb(1., 1., 1.)),
            },
            ..Default::default()
        })
        .insert(PanCam {
            grab_buttons: vec![MouseButton::Left], // which buttons should drag the camera
            enabled: true, // when false, controls are disabled. See toggle example.
            zoom_to_cursor: true, // whether to zoom towards the mouse or the center of the screen
            min_scale: 1., // prevent the camera from zooming too far in
            max_scale: Some(40.), // prevent the camera from zooming too far out
            ..Default::default()
        });
}
