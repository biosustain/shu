use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::prelude::*;
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_prototype_lyon::prelude::*;

mod escher;
mod geom;

use escher::{CustomAssetLoader, EscherMap, EscherPlugin, MapState, Metabolite, Reaction};

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(PanCamPlugin::default())
        .add_plugin(ShapePlugin)
        .add_plugin(EscherPlugin)
        .add_startup_system(setup_system)
        .run();
}

fn setup_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let escher_handle: Handle<EscherMap> = asset_server.load("ecoli_core_map.json");
    commands.insert_resource(MapState {
        escher_map: escher_handle,
        loaded: false,
    });

    commands
        .spawn(Camera2dBundle {
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::Custom(Color::rgb(1., 1., 1.)),
            },
            ..Default::default()
        })
        .insert(PanCam::default());
}
