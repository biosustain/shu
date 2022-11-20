use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::BoxedFuture,
};
/// Data model of escher JSON maps
/// TODO: borrow strings
use serde::Deserialize;
use serde_json;
use std::collections::HashMap;

#[derive(Deserialize, TypeUuid)]
#[uuid = "413be529-bfeb-41b3-9db0-4b8b380a2c46"]
pub struct EscherMap {
    info: EscherInfo,
    metabolism: Metabolism,
}

#[derive(Default)]
pub struct CustomAssetLoader;

impl AssetLoader for CustomAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let custom_asset = serde_json::from_slice::<EscherMap>(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(custom_asset));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}

impl EscherMap {
    pub fn get_components(&self) -> (Vec<Reaction>, Vec<Metabolite>) {
        (
            self.metabolism.reactions.clone().into_values().collect(),
            self.metabolism
                .nodes
                .clone()
                .into_iter()
                .filter_map(|(_, met)| match met {
                    Node::Metabolite(met) => Some(met),
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
}

#[derive(Deserialize)]
struct EscherInfo {
    map_name: String,
    map_id: String,
    map_description: String,
    homepage: String,
    schema: String,
}

#[derive(Deserialize)]
struct Metabolism {
    reactions: HashMap<u64, Reaction>,
    nodes: HashMap<u64, Node>,
}

/// Component for Bevy that will be rendered on screen.
/// Rendered as arrow.
#[derive(Component, Deserialize, Clone)]
pub struct Reaction {
    name: String,
    pub bigg_id: String,
    reversibility: bool,
    label_x: f32,
    label_y: f32,
    gene_reaction_rule: String,
    genes: Vec<HashMap<String, String>>,
    metabolites: Vec<MetRef>,
    pub segments: HashMap<u32, Segment>,
}

#[derive(Deserialize, Clone)]
struct MetRef {
    coefficient: f32,
    bigg_id: String,
}

#[derive(Deserialize, Clone)]
pub struct Segment {
    pub from_node_id: String,
    pub to_node_id: String,
    pub b1: Option<BezierHandle>,
    pub b2: Option<BezierHandle>,
}

#[derive(Deserialize, Clone)]
pub struct BezierHandle {
    pub x: f32,
    pub y: f32,
}

#[derive(Deserialize, Clone)]
#[serde(tag = "node_type", rename_all = "lowercase")]
enum Node {
    Metabolite(Metabolite),
    Multimarker { x: f32, y: f32 },
    Midmarker { x: f32, y: f32 },
}

/// Component for Bevy that will be rendered on screen.
/// Rendered as circles.
#[derive(Component, Deserialize, Clone)]
pub struct Metabolite {
    pub x: f32,
    pub y: f32,
    label_x: f32,
    label_y: f32,
    name: String,
    pub bigg_id: String,
    pub node_is_primary: bool,
}
