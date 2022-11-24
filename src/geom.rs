use bevy::prelude::Component;

/// When in a Entity with `Aesthetics`, it will plot whatever aes to
/// the arrows in the map.
#[derive(Component)]
pub struct GeomArrow {
    pub plotted: bool,
}

#[derive(Clone)]
pub enum Side {
    Left,
    Right,
    // for hovering instances
    Up,
}

/// When in a Entity with `Aesthetics`, it will plot whatever aes to
/// the a KDE on the side of the arrows in the map..
#[derive(Component)]
pub struct GeomKde {
    pub side: Side,
}
impl GeomKde {
    pub fn left() -> Self {
        Self { side: Side::Left }
    }
    pub fn right() -> Self {
        Self { side: Side::Right }
    }
}

/// When in a Entity with `Aesthetics`, it will plot whatever aes to
/// the a KDE on the side of the arrows in the map..
#[derive(Component, Clone)]
pub struct GeomHist {
    pub side: Side,
    pub rendered: bool,
}
impl GeomHist {
    pub fn left() -> Self {
        Self {
            side: Side::Left,
            rendered: false,
        }
    }
    pub fn right() -> Self {
        Self {
            side: Side::Right,
            rendered: false,
        }
    }
    pub fn up() -> Self {
        Self {
            side: Side::Up,
            rendered: false,
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
}

/// Component of all popups.
#[derive(Component)]
pub struct PopUp;

/// Component of all popups.
#[derive(Component)]
pub struct AnyTag {
    pub id: u64,
}
