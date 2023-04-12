//! Module to handle dynamic scaling on zoom.
use crate::funcplot::lerp;
use bevy::prelude::*;

/// Constant that matches bevy_pancman Line pixel increment
pub struct ZoomPlugin;

impl Plugin for ZoomPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(zoom_fonts);
    }
}

#[derive(Component)]
pub struct DefaultFontSize {
    pub size: f32,
}

/// Rerender fonts on zoom to achieve a constantly-readable size.
fn zoom_fonts(
    mut text_query: Query<(&mut Text, &DefaultFontSize)>,
    proj_query: Query<&OrthographicProjection, (Changed<Transform>, Without<DefaultFontSize>)>,
) {
    let Ok(proj) = proj_query.get_single() else {return};
    for (mut text, def) in text_query.iter_mut() {
        for mut section in text.sections.iter_mut() {
            let new_font_size = lerp(proj.scale, 1., 40., def.size, def.size * 10.);
            // step update to enhance perfomance
            if (new_font_size - section.style.font_size).abs() > 1.0 {
                section.style.font_size = new_font_size;
            }
        }
    }
}
