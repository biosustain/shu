use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::prelude::*;
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_prototype_lyon::prelude::*;

mod escher;
mod geom;

use escher::{BezierHandle, CustomAssetLoader, EscherMap, Metabolite, Reaction};
use geom::{GeomMetabolite, GeomReaction};
use serde_json;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_asset::<EscherMap>()
        .add_plugin(PanCamPlugin::default())
        .init_asset_loader::<CustomAssetLoader>()
        .add_plugin(ShapePlugin)
        .add_startup_system(setup_system)
        .add_system(load_map)
        .run();
}

#[derive(Resource)]
struct MapState {
    escher_map: Handle<EscherMap>,
    loaded: bool,
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

/// Load escher map once the asset is available.
/// The colors correspond to the default escher colors.
fn load_map(
    mut commands: Commands,
    mut state: ResMut<MapState>,
    mut custom_assets: ResMut<Assets<EscherMap>>,
) {
    let custom_asset = custom_assets.get_mut(&mut state.escher_map);
    if state.loaded || custom_asset.is_none() {
        return;
    }
    let my_map = custom_asset.unwrap();
    let (reactions, metabolites) = my_map.get_components();
    // center all metabolites positions
    let (total_x, total_y) = metabolites
        .iter()
        .map(|met| (met.x, met.y))
        .fold((0., 0.), |(acc_x, acc_y), (x, y)| (acc_x + x, acc_y + y));
    let (center_x, center_y) = (
        total_x / metabolites.len() as f32,
        total_y / metabolites.len() as f32,
    );
    for met in metabolites {
        let shape = shapes::RegularPolygon {
            sides: 6,
            feature: shapes::RegularPolygonFeature::Radius(if met.node_is_primary {
                20.0
            } else {
                10.0
            }),
            ..shapes::RegularPolygon::default()
        };
        commands
            .spawn(GeometryBuilder::build_as(
                &shape,
                DrawMode::Outlined {
                    fill_mode: FillMode::color(Color::rgb(224. / 255., 137. / 255., 101. / 255.)),
                    outline_mode: StrokeMode::new(
                        Color::rgb(162. / 255., 69. / 255., 16. / 255.),
                        4.0,
                    ),
                },
                Transform::from_xyz(met.x - center_x, -met.y + center_y, 1.),
            ))
            .insert(GeomMetabolite { id: met.bigg_id });
    }
    for reac in reactions {
        for (_, segment) in reac.segments {
            if let (Some(from), Some(to)) = (
                my_map.met_coords(&segment.from_node_id),
                my_map.met_coords(&segment.to_node_id),
            ) {
                let mut path_builder = PathBuilder::new();
                path_builder.move_to(Vec2::ZERO);
                match (segment.b1, segment.b2) {
                    (Some(BezierHandle { x, y }), None) | (None, Some(BezierHandle { x, y })) => {
                        path_builder.quadratic_bezier_to(
                            Vec2::new(x - from.x, -y + from.y),
                            Vec2::new(to.x - from.x, -to.y + from.y),
                        );
                    }
                    (Some(BezierHandle { x: x1, y: y1 }), Some(BezierHandle { x: x2, y: y2 })) => {
                        path_builder.cubic_bezier_to(
                            Vec2::new(x1 - from.x, -y1 + from.y),
                            Vec2::new(x2 - from.x, -y2 + from.y),
                            Vec2::new(to.x - from.x, -to.y + from.y),
                        );
                    }
                    (None, None) => {
                        let v = Vec2::new(to.x - from.x, -to.y + from.y);
                        path_builder.line_to(v);
                    }
                }
                let line = path_builder.build();
                commands
                    .spawn(GeometryBuilder::build_as(
                        &line,
                        DrawMode::Stroke(StrokeMode::new(
                            Color::rgb(51. / 255., 78. / 255., 101. / 255.),
                            10.0,
                        )),
                        Transform::from_xyz(from.x - center_x, -from.y + center_y, 0.),
                    ))
                    .insert(GeomReaction {
                        id: reac.bigg_id.clone(),
                    });
            }
        }
    }
    info!("Map loaded!");

    state.loaded = true;
}
