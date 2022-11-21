use bevy::prelude::Component;

/// When in a Entity with `Aesthetics`, it will plot whatever aes to
/// the arrows in the map.
#[derive(Component)]
pub struct GeomArrow {
    pub plotted: bool,
}

/// When in a Entity with `Aesthetics`, it will plot whatever aes to
/// the circles in the map.
#[derive(Component)]
pub struct GeomMetabolite {
    plotted: bool,
}
