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
        // .add_startup_system(setup_demo_arrow_size)
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
        .insert(PanCam::default());
}

// System for testing.
// fn _setup_demo_arrow_size(mut commands: Commands) {
//     commands.spawn((
//         aesthetics::Aesthetics {
//             plotted: false,
//             identifiers: vec![String::from("PFK"), String::from("GAPD")],
//         },
//         aesthetics::Gsize {},
//         aesthetics::Point(vec![40f32, 20f32]),
//         geom::GeomArrow { plotted: false },
//     ));
//     commands.spawn((
//         aesthetics::Aesthetics {
//             plotted: false,
//             identifiers: vec![String::from("FUM")],
//         },
//         aesthetics::Gsize {},
//         aesthetics::Distribution(vec![vec![20f32, 40f32]]),
//         geom::GeomArrow { plotted: false },
//     ));
//     commands.spawn((
//         aesthetics::Aesthetics {
//             plotted: false,
//             identifiers: vec![String::from("PFK"), String::from("GAPD")],
//         },
//         aesthetics::Gcolor {},
//         aesthetics::Point(vec![180f32, 30f32]),
//         geom::GeomArrow { plotted: false },
//     ));
//     warn!("Some Aes loaded! Remove this setup system when not debugging");
// }
