use crate::aesthetics::{AesPlugin, Aesthetics, Distribution, Gy, Point, Unscale};
use crate::geom::{GeomHist, HistTag, Xaxis};
use crate::gui::UiState;
use crate::{escher, geom};
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::{DrawMode, GeometryBuilder, PathBuilder, StrokeMode};

use bevy::asset::FileAssetIo;
use bevy::tasks::IoTaskPool;

/// Setup to test systems that require [`AsserServer`] as an argument.
/// Adapted form bevy source code.
fn setup(asset_path: impl AsRef<std::path::Path>) -> AssetServer {
    IoTaskPool::init(Default::default);

    AssetServer::new(FileAssetIo::new(asset_path, false))
}

#[test]
fn gy_dist_aes_spaws_xaxis_spawns_hist() {
    // Setup app
    let mut app = App::new();
    // build_axes queries for aesthetics
    app.world
        .spawn(Aesthetics {
            plotted: false,
            identifiers: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            condition: None,
        })
        .insert(Gy {})
        .insert(Distribution(vec![
            vec![1f32, 2., 2.],
            vec![1f32, 2., 1.],
            vec![6f32, 2., 6.],
        ]))
        .insert(GeomHist::right(geom::HistPlot::Kde));
    // and for Paths with ArrowTag
    let path_builder = PathBuilder::new();
    let line = path_builder.build();
    app.world.spawn((
        GeometryBuilder::build_as(
            &line,
            DrawMode::Stroke(StrokeMode::new(
                Color::rgb(51. / 255., 78. / 255., 101. / 255.),
                10.0,
            )),
            Transform::from_xyz(1., 1., 1.),
        ),
        escher::ArrowTag {
            id: String::from("a"),
            hists: None,
            node_id: 9,
            direction: Vec2::new(0., 1.),
        },
    ));

    let asset_server = setup("assets");
    app.insert_resource(asset_server);
    app.insert_resource(UiState::default());
    app.add_plugin(AesPlugin);
    app.update();

    // one update for xaxis creation
    assert!(app
        .world
        .query::<&Xaxis>()
        .iter(&app.world)
        .next()
        .is_some());

    // another update for HistTag creation
    app.update();
    assert!(app
        .world
        .query::<(&HistTag, &Path)>()
        .iter(&app.world)
        .next()
        .is_some());
}

#[test]
fn point_dist_aes_spaws_box_axis_spawns_box() {
    // Setup app
    let mut app = App::new();
    // build_axes queries for aesthetics
    app.world
        .spawn(Aesthetics {
            plotted: false,
            identifiers: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            condition: None,
        })
        .insert(Gy {})
        .insert(Point(vec![1f32, 2., 2.]))
        .insert(GeomHist::right(geom::HistPlot::Kde));
    // and for Paths with ArrowTag
    let path_builder = PathBuilder::new();
    let line = path_builder.build();
    app.world.spawn((
        GeometryBuilder::build_as(
            &line,
            DrawMode::Stroke(StrokeMode::new(
                Color::rgb(51. / 255., 78. / 255., 101. / 255.),
                10.0,
            )),
            Transform::from_xyz(1., 1., 1.),
        ),
        escher::ArrowTag {
            id: String::from("a"),
            hists: None,
            node_id: 9,
            direction: Vec2::new(0., 1.),
        },
    ));

    app.insert_resource(UiState::default());
    app.insert_resource(AssetServer::new(FileAssetIo::new("asset1", false)));
    app.add_plugin(AesPlugin);
    app.update();

    assert!(app
        .world
        .query::<(&Xaxis, &Unscale)>()
        .iter(&app.world)
        .next()
        .is_some());

    // another update for HistTag creation
    app.update();

    assert!(app
        .world
        .query::<(&HistTag, &Unscale, &Path)>()
        .iter(&app.world)
        .next()
        .is_some());
}

#[test]
fn point_dist_aes_spawns_side_box() {
    // Setup app
    let mut app = App::new();
    // build_axes queries for aesthetics
    app.world
        .spawn(Aesthetics {
            plotted: false,
            identifiers: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            condition: None,
        })
        .insert(Gy {})
        .insert(Point(vec![1f32, 2., 2.]))
        .insert(GeomHist::right(geom::HistPlot::Kde));
    // and for Paths with ArrowTag
    let path_builder = PathBuilder::new();
    let line = path_builder.build();
    app.world.spawn((
        GeometryBuilder::build_as(
            &line,
            DrawMode::Stroke(StrokeMode::new(
                Color::rgb(51. / 255., 78. / 255., 101. / 255.),
                10.0,
            )),
            Transform::from_xyz(1., 1., 1.),
        ),
        escher::ArrowTag {
            id: String::from("a"),
            hists: None,
            node_id: 9,
            direction: Vec2::new(0., 1.),
        },
    ));
    app.insert_resource(UiState::default());
    app.insert_resource(AssetServer::new(FileAssetIo::new("asset2", false)));

    app.add_plugin(AesPlugin);
    // one update for xaxis creation
    app.update();
    // another update for histtag creation
    app.update();

    assert!(app
        .world
        .query::<&HistTag>()
        .iter(&app.world)
        .next()
        .is_some());
}
