use bevy::prelude::Component;

/// When in a Entity with `Aesthetics`, it will plot whatever aes to
/// the arrows in the map.
#[derive(Component)]
pub struct GeomArrow {
    pub plotted: bool,
}

pub enum Side {
    Left,
    Right,
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
#[derive(Component)]
pub struct GeomHist {
    pub side: Side,
}
impl GeomHist {
    pub fn left() -> Self {
        Self { side: Side::Left }
    }
    pub fn right() -> Self {
        Self { side: Side::Right }
    }
}

/// When in a Entity with `Aesthetics`, it will plot whatever aes to
/// the circles in the map.
#[derive(Component)]
pub struct GeomMetabolite {
    pub plotted: bool,
}
