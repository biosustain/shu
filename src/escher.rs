//! Data model of escher JSON maps
//! TODO: borrow strings
use crate::geom::{GeomHist, HistTag, Side, Xaxis};
use bevy::{prelude::*, reflect::TypeUuid};
use bevy_prototype_lyon::prelude::*;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::HashMap};

pub const ARROW_COLOR: Color = Color::rgb(95. / 255., 94. / 255., 95. / 255.);
pub const MET_COLOR: Color = Color::rgb(190. / 255., 185. / 255., 185. / 255.);
pub const MET_STROK: Color = Color::rgb(95. / 255., 94. / 255., 95. / 255.);

pub struct EscherPlugin;

impl Plugin for EscherPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NodeToText::default())
            .add_system(load_map);
    }
}

#[derive(Resource)]
pub struct MapState {
    pub escher_map: Handle<EscherMap>,
    pub loaded: bool,
}

/// Resource to map arrow ids to their [`Entity`] for hovering purposes.
#[derive(Resource, Default)]
pub struct NodeToText {
    pub inner: HashMap<u64, Entity>,
}

#[derive(Deserialize, TypeUuid, Default, Serialize)]
#[uuid = "413be529-bfeb-41b3-9db0-4b8b380a2c46"]
pub struct EscherMap {
    #[allow(dead_code)]
    info: EscherInfo,
    pub metabolism: Metabolism,
}

impl EscherMap {
    pub fn get_components(&self) -> (HashMap<u64, Reaction>, HashMap<u64, Metabolite>) {
        (
            self.metabolism.reactions.clone(),
            self.metabolism
                .nodes
                .clone()
                .into_iter()
                .filter_map(|(id, met)| match met {
                    Node::Metabolite(met) => Some((id, met)),
                    _ => None,
                })
                .collect(),
        )
    }

    /// Get the coordinates of a metabolite given a node id
    pub fn met_coords(&self, met_id: &str) -> Option<Vec2> {
        let met = self.metabolism.nodes.get(&met_id.parse().unwrap())?;
        match met {
            Node::Metabolite(Metabolite { x, y, .. })
            | Node::Multimarker { x, y }
            | Node::Midmarker { x, y } => Some(Vec2::new(*x, *y)),
        }
    }

    /// Reaction direction as defined by the vector that follows the longest segment.
    /// This is needed to calculate rotation angles for elements at the side of the
    /// reactions.
    pub fn main_direction(&self, reac: &Reaction) -> Vec2 {
        reac.segments
            .values()
            .filter_map(|seg| {
                self.metabolism
                    .nodes
                    .get(&seg.from_node_id.parse().unwrap())
            })
            .chain(
                reac.segments
                    .values()
                    .filter_map(|seg| self.metabolism.nodes.get(&seg.to_node_id.parse().unwrap())),
            )
            .filter_map(|node| match node {
                Node::Metabolite(Metabolite {
                    x,
                    y,
                    node_is_primary,
                    ..
                }) if *node_is_primary => Some(Vec2::new(*x, *y)),
                _ => None,
            })
            .combinations(2)
            .map(|vec| vec[1] - vec[0])
            // avoid zero vectors
            .filter(|vec| vec.max_element() > 1e-5)
            .max_by(|x, y| {
                if x.length() - y.length() > 1e-5 {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            })
            .unwrap_or(Vec2::Y)
            .normalize()
    }
}

#[derive(Deserialize, Serialize, Default)]
struct EscherInfo {
    map_name: String,
    map_id: String,
    map_description: String,
    homepage: String,
    schema: String,
}

#[derive(Deserialize, Serialize, Default)]
pub struct Metabolism {
    pub reactions: HashMap<u64, Reaction>,
    nodes: HashMap<u64, Node>,
}

/// DeSerializable representation of Transform to store histogram positions.
#[derive(Component, Deserialize, Serialize, Clone)]
pub struct SerTransform {
    translation: Vec3,
    rotation: [f32; 4],
    scale: Vec3,
}

impl From<Transform> for SerTransform {
    fn from(transform: Transform) -> Self {
        Self {
            translation: transform.translation,
            rotation: transform.rotation.to_array(),
            scale: transform.scale,
        }
    }
}

impl From<SerTransform> for Transform {
    fn from(transform: SerTransform) -> Self {
        Self {
            translation: transform.translation,
            rotation: Quat::from_vec4(transform.rotation.into()),
            scale: transform.scale,
        }
    }
}

/// Component for Bevy that will be rendered on screen.
/// Rendered as arrow.
#[derive(Component, Deserialize, Serialize, Clone)]
pub struct Reaction {
    name: String,
    pub bigg_id: String,
    reversibility: bool,
    label_x: f32,
    label_y: f32,
    gene_reaction_rule: String,
    pub hist_position: Option<HashMap<Side, SerTransform>>,
    // genes: Vec<HashMap<String, String>>,
    metabolites: Vec<MetRef>,
    pub segments: HashMap<u32, Segment>,
}

#[derive(Deserialize, Serialize, Clone)]
struct MetRef {
    coefficient: f32,
    bigg_id: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Segment {
    pub from_node_id: String,
    pub to_node_id: String,
    pub b1: Option<BezierHandle>,
    pub b2: Option<BezierHandle>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct BezierHandle {
    pub x: f32,
    pub y: f32,
}

#[derive(Deserialize, Clone, Serialize)]
#[serde(tag = "node_type", rename_all = "lowercase")]
enum Node {
    Metabolite(Metabolite),
    Multimarker { x: f32, y: f32 },
    Midmarker { x: f32, y: f32 },
}

/// Component for Bevy that will be rendered on screen.
/// Rendered as circles.
#[derive(Component, Deserialize, Clone, Serialize)]
pub struct Metabolite {
    pub x: f32,
    pub y: f32,
    label_x: f32,
    label_y: f32,
    name: String,
    pub bigg_id: String,
    pub node_is_primary: bool,
}

/// Component to differentiate circles via identifier (bigg_id in [`Metabolite`]).
#[derive(Component, Deserialize, Clone)]
pub struct CircleTag {
    pub id: String,
}
/// Component to differentiate arrows via identifier (bigg_id in [`Reaction`]).
#[derive(Component, Deserialize, Clone)]
pub struct ArrowTag {
    pub id: String,
    pub direction: Vec2,
    pub node_id: u64,
    pub hists: Option<HashMap<Side, SerTransform>>,
}

pub trait Tag: Component {
    fn id(&self) -> &str;
}

impl Tag for CircleTag {
    fn id(&self) -> &str {
        &self.id
    }
}

impl Tag for ArrowTag {
    fn id(&self) -> &str {
        &self.id
    }
}

pub trait Labelled {
    fn label_position(&self) -> Vec2;
    fn id(&mut self) -> String;
}

fn build_text_tag(
    node: &mut impl Labelled,
    font: Handle<Font>,
    center_x: f32,
    center_y: f32,
    font_size: f32,
) -> Text2dBundle {
    let pos = node.label_position();
    let text = Text::from_section(
        node.id(),
        TextStyle {
            font,
            font_size,
            color: ARROW_COLOR,
        },
    )
    .with_alignment(TextAlignment::CENTER_LEFT);
    Text2dBundle {
        text,
        transform: Transform::from_xyz(pos.x - center_x, -pos.y + center_y, 4.0),
        ..default()
    }
}

impl Labelled for Metabolite {
    fn label_position(&self) -> Vec2 {
        Vec2::new(self.label_x, self.label_y)
    }

    fn id(&mut self) -> String {
        std::mem::take(&mut self.bigg_id)
    }
}

impl Labelled for Reaction {
    fn label_position(&self) -> Vec2 {
        Vec2::new(self.label_x, self.label_y)
    }

    fn id(&mut self) -> String {
        std::mem::take(&mut self.bigg_id)
    }
}

/// Mark an entity as hoverable.
#[derive(Component)]
pub struct Hover {
    pub id: String,
    pub node_id: u64,
    pub xlimits: Option<(f32, f32)>,
}

/// Load escher map once the asset is available.
/// The colors correspond to the default escher colors.
pub fn load_map(
    mut commands: Commands,
    mut state: ResMut<MapState>,
    mut node_to_text: ResMut<NodeToText>,
    asset_server: Res<AssetServer>,
    mut custom_assets: ResMut<Assets<EscherMap>>,
    existing_map: Query<Entity, Or<(With<CircleTag>, With<ArrowTag>, With<HistTag>, With<Xaxis>)>>,
    mut existing_geom_hist: Query<&mut GeomHist>,
) {
    let custom_asset = custom_assets.get_mut(&state.escher_map);
    if state.loaded || custom_asset.is_none() {
        return;
    }
    let node_to_text = &mut node_to_text.inner;

    // previous arrows and circles are despawned.
    // HistTags has to be despawned too because they are spawned when painted,
    // but they will be repainted at the end of loading the amp
    for e in existing_map.iter() {
        commands.entity(e).despawn_recursive();
    }

    let my_map = custom_asset.unwrap();
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let (reactions, metabolites) = my_map.get_components();
    // center all metabolites positions
    let (total_x, total_y) = metabolites
        .values()
        .map(|met| (met.x, met.y))
        .fold((0., 0.), |(acc_x, acc_y), (x, y)| (acc_x + x, acc_y + y));
    let (center_x, center_y) = (
        total_x / metabolites.len() as f32,
        total_y / metabolites.len() as f32,
    );
    for (node_id, mut met) in metabolites {
        let shape = shapes::RegularPolygon {
            sides: 6,
            feature: shapes::RegularPolygonFeature::Radius(if met.node_is_primary {
                20.0
            } else {
                10.0
            }),
            ..shapes::RegularPolygon::default()
        };
        let circle = CircleTag {
            id: met.bigg_id.clone(),
        };
        let hover = Hover {
            id: met.bigg_id.clone(),
            node_id,
            xlimits: None,
        };
        commands.spawn((
            GeometryBuilder::build_as(
                &shape,
                DrawMode::Outlined {
                    fill_mode: FillMode::color(MET_COLOR),
                    outline_mode: StrokeMode::new(MET_STROK, 4.0),
                },
                Transform::from_xyz(met.x - center_x, -met.y + center_y, 2.),
            ),
            circle.clone(),
        ));
        commands.spawn((
            build_text_tag(&mut met, font.clone(), center_x, center_y, 25.),
            hover,
            circle,
        ));
    }
    for (node_id, mut reac) in reactions {
        let mut path_builder = PathBuilder::new();
        // origin of the figure as the center of mass
        let ori: Vec2 = reac
            .segments
            .values()
            .map(|seg| {
                (
                    my_map.met_coords(&seg.from_node_id),
                    my_map.met_coords(&seg.to_node_id),
                )
            })
            .filter_map(|(from, to)| match (from, to) {
                (Some(f), Some(t)) => Some(f + t),
                _ => None,
            })
            .sum::<Vec2>()
            / (2. * reac.segments.len() as f32);
        let direction = my_map.main_direction(&reac);
        for (_, segment) in reac.segments.iter_mut() {
            if let (Some(from), Some(to)) = (
                my_map.met_coords(&segment.from_node_id),
                my_map.met_coords(&segment.to_node_id),
            ) {
                path_builder.move_to(Vec2::new(from.x - ori.x, -from.y + ori.y));
                match (
                    std::mem::take(&mut segment.b1),
                    std::mem::take(&mut segment.b2),
                ) {
                    (Some(BezierHandle { x, y }), None) | (None, Some(BezierHandle { x, y })) => {
                        path_builder.quadratic_bezier_to(
                            Vec2::new(x - ori.x, -y + ori.y),
                            Vec2::new(to.x - ori.x, -to.y + ori.y),
                        );
                    }
                    (Some(BezierHandle { x: x1, y: y1 }), Some(BezierHandle { x: x2, y: y2 })) => {
                        path_builder.cubic_bezier_to(
                            Vec2::new(x1 - ori.x, -y1 + ori.y),
                            Vec2::new(x2 - ori.x, -y2 + ori.y),
                            Vec2::new(to.x - ori.x, -to.y + ori.y),
                        );
                    }
                    (None, None) => {
                        let v = Vec2::new(to.x - ori.x, -to.y + ori.y);
                        path_builder.line_to(v);
                    }
                }
            }
        }
        let line = path_builder.build();
        let arrow = ArrowTag {
            id: reac.bigg_id.clone(),
            hists: reac.hist_position.clone(),
            node_id,
            direction,
        };
        let hover = Hover {
            id: reac.bigg_id.clone(),
            node_id,
            xlimits: None,
        };
        commands.spawn((
            GeometryBuilder::build_as(
                &line,
                DrawMode::Stroke(StrokeMode::new(ARROW_COLOR, 10.0)),
                Transform::from_xyz(ori.x - center_x, -ori.y + center_y, 1.),
            ),
            arrow.clone(),
        ));
        // spawn the text and collect its id in the hashmap for hovering.
        node_to_text.insert(
            node_id,
            commands
                .spawn((
                    build_text_tag(&mut reac, font.clone(), center_x, center_y, 35.),
                    arrow,
                    hover,
                ))
                .id(),
        );
    }
    // Send signal to repaint histograms.
    for mut geom in existing_geom_hist.iter_mut() {
        geom.rendered = false;
        geom.in_axis = false;
    }
    info!("Map loaded!");
    state.loaded = true;
}
