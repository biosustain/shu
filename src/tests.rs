//! Unit testing on app-updates.
use crate::aesthetics::{AesPlugin, Aesthetics, Distribution, Gy, Point, RestoreEvent, Unscale};
use crate::geom::{AesFilter, GeomHist, HistTag, Xaxis, YCategory};
use crate::gui::{file_drop, ActiveData, UiState};
use crate::{data, escher, geom, info};
use bevy::prelude::*;
use bevy::time::TimePlugin;
use bevy_prototype_lyon::prelude::{GeometryBuilder, PathBuilder, Shape, Stroke};

use bevy::tasks::IoTaskPool;

/// Setup to test systems that require [`AsserServer`] as an argument.
/// Adapted form bevy source code.
fn setup(app: &mut App, asset_path: &str) {
    IoTaskPool::get_or_init(Default::default);
    let asset_plug = AssetPlugin {
        file_path: asset_path.to_string(),
        ..Default::default()
    };
    app.add_plugins(asset_plug);
}

#[test]
fn gy_dist_aes_spaws_xaxis_spawns_hist() {
    // Setup app
    let mut app = App::new();
    setup(&mut app, "assets");
    app.world().get_resource::<AssetServer>().unwrap();
    app.init_asset::<Font>();
    // build_axes queries for aesthetics
    app.world_mut()
        .spawn(Aesthetics {
            identifiers: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            condition: None,
        })
        .insert(Gy {})
        .insert(Distribution(vec![
            vec![1f32, 2., 2.],
            vec![1f32, 2., 1.],
            vec![6f32, 2., 6.],
        ]))
        .insert(AesFilter {})
        .insert(GeomHist::right(geom::HistPlot::Kde));
    // and for Paths with ArrowTag
    let path_builder = PathBuilder::new();
    let line = path_builder.build();
    app.world_mut().spawn((
        GeometryBuilder::build_as(&line),
        Transform::from_xyz(1., 1., 1.),
        Stroke::new(Color::srgb(51. / 255., 78. / 255., 101. / 255.), 10.0),
        escher::ArrowTag {
            id: String::from("a"),
            hists: None,
            node_id: 9,
            direction: Vec2::new(0., 1.),
        },
        AesFilter {},
    ));

    app.insert_resource(ActiveData::default());
    app.insert_resource(UiState::default());
    app.add_plugins(AesPlugin);
    app.update();

    // one update for xaxis creation
    assert!(app
        .world_mut()
        .query::<&Xaxis>()
        .iter(app.world())
        .next()
        .is_some());

    // another update for HistTag creation
    app.update();
    assert!(app
        .world_mut()
        .query::<(&HistTag, &Shape)>()
        .iter(app.world())
        .next()
        .is_some());
}

#[test]
fn point_dist_aes_spaws_box_axis_spawns_box() {
    // Setup app
    let mut app = App::new();
    setup(&mut app, "assets");
    app.world().get_resource::<AssetServer>().unwrap();
    app.init_asset::<Font>();
    // build_axes queries for aesthetics
    app.world_mut()
        .spawn(Aesthetics {
            identifiers: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            condition: None,
        })
        .insert(Gy {})
        .insert(Point(vec![1f32, 2., 2.]))
        .insert(AesFilter {})
        .insert(YCategory {
            idx: vec![0, 1, 2],
            tags: vec![
                Some(String::from("a")),
                Some(String::from("b")),
                Some(String::from("c")),
            ],
        })
        .insert(GeomHist::right(geom::HistPlot::Kde));
    // and for Paths with ArrowTag
    let path_builder = PathBuilder::new();
    let line = path_builder.build();
    app.world_mut().spawn((
        GeometryBuilder::build_as(&line),
        Transform::from_xyz(1., 1., 1.),
        Stroke::new(Color::srgb(51. / 255., 78. / 255., 101. / 255.), 10.0),
        escher::ArrowTag {
            id: String::from("a"),
            hists: None,
            node_id: 9,
            direction: Vec2::new(0., 1.),
        },
        AesFilter {},
    ));

    app.insert_resource(UiState::default());
    app.insert_resource(ActiveData::default());
    app.add_plugins(AesPlugin);
    app.update();

    assert!(app
        .world_mut()
        .query::<(&Xaxis, &Unscale)>()
        .iter(app.world())
        .next()
        .is_some());

    // another update for HistTag creation
    app.update();

    assert!(app
        .world_mut()
        .query::<(&HistTag, &Unscale, &Shape)>()
        .iter(app.world())
        .next()
        .is_some());
}

#[test]
fn loading_file_drop_does_not_crash() {
    // Setup app
    let mut app = App::new();
    app.insert_resource(UiState::default());
    app.add_event::<RestoreEvent>();
    setup(&mut app, "assets");
    app.insert_resource(data::ReactionState {
        reaction_data: None,
        loaded: false,
    });
    app.add_plugins(TimePlugin);
    app.add_plugins(info::InfoPlugin);
    app.add_event::<FileDragAndDrop>();
    app.add_plugins(data::DataPlugin);
    app.add_plugins(escher::EscherPlugin);
    app.init_asset::<Font>();
    let asset_server = app.world().get_resource::<AssetServer>().unwrap();
    let escher_handle: Handle<escher::EscherMap> = asset_server.load("ecoli_core_map.json");
    app.insert_resource(escher::MapState {
        escher_map: escher_handle,
        loaded: false,
    });
    app.add_systems(Update, file_drop);

    app.update();
    app.world_mut().send_event(FileDragAndDrop::DroppedFile {
        window: Entity::from_raw(24),
        path_buf: "assets/ecoli_core_map.json".into(),
    });
    app.update();
}
