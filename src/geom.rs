use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

/// When in a Entity with `Aesthetics`, it will plot whatever aes to
/// the arrows in the map.
#[derive(Component)]
pub struct GeomArrow {
    pub plotted: bool,
}

#[derive(Hash, PartialEq, Eq, Debug, Clone, Deserialize, Serialize)]
pub enum Side {
    Left,
    Right,
    // for hovering instances
    Up,
}

#[derive(Debug, Clone)]
pub enum HistPlot {
    Hist,
    Kde,
    // Point estimate.
    BoxPoint,
}

/// When in a Entity with `Aesthetics`, it will plot whatever aes to
/// a histogram/KDE/box on the side of the arrows in the map.
#[derive(Component, Clone, Debug)]
pub struct GeomHist {
    pub side: Side,
    pub rendered: bool,
    pub mean: Option<f32>,

    pub in_axis: bool,
    pub plot: HistPlot,
}

impl GeomHist {
    pub fn left(plot: HistPlot) -> Self {
        Self {
            side: Side::Left,
            rendered: false,
            in_axis: false,
            mean: None,
            plot,
        }
    }
    pub fn right(plot: HistPlot) -> Self {
        Self {
            side: Side::Right,
            rendered: false,
            mean: None,
            in_axis: false,
            plot,
        }
    }
    pub fn up(plot: HistPlot) -> Self {
        Self {
            side: Side::Up,
            rendered: false,
            in_axis: false,
            mean: None,
            plot,
        }
    }
}

/// When in a Entity with `Aesthetics`, it will plot whatever aes to
/// the circles in the map.
#[derive(Component)]
pub struct GeomMetabolite {
    pub plotted: bool,
}

/// Component applied to all Hist-like entities (spawned by a GeomKde, GeomHist, etc. aesthetic)
/// This allow us to query for systems like normalize or drag.
#[derive(Component)]
pub struct HistTag {
    pub side: Side,
    pub condition: Option<String>,
    pub node_id: u64,
}

/// Component that indicates the plot position and axis.
#[derive(Debug, Component)]
pub struct Xaxis {
    pub id: String,
    pub arrow_size: f32,
    pub xlimits: (f32, f32),
    pub side: Side,
    pub plot: HistPlot,
    pub node_id: u64,
    pub conditions: Vec<String>,
}

/// Component that marks something susceptible of being dragged/rotated.
#[derive(Debug, Component, Default)]
pub struct Drag {
    pub dragged: bool,
    pub rotating: bool,
}

impl std::fmt::Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Side::Right => "right",
                Side::Left => "left",
                Side::Up => "up",
            }
        )
    }
}

impl std::fmt::Display for Xaxis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Xaxis = [ id = {}, side = {} ]", self.id, self.side)
    }
}

/// Component of all popups.
#[derive(Component)]
pub struct PopUp;

/// Component of all popups.
#[derive(Component, Debug)]
pub struct AnyTag {
    pub id: u64,
}
