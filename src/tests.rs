use crate::aesthetics::{AesPlugin, Aesthetics, Gy, Point, Unscale};
use crate::geom::{GeomHist, HistTag, Xaxis};
use crate::gui::UiState;
use crate::{escher, geom};
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::{DrawMode, GeometryBuilder, PathBuilder, StrokeMode};

use bevy::asset::FileAssetIo;

#[test]
fn point_dist_aes_spaws_box_axis() {
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
