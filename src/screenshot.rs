use crate::{escher::MapDimensions, info::Info};
use bevy::prelude::*;
use bevy::render::view::screenshot::ScreenshotManager;
use bevy::window::PrimaryWindow;
use bevy_prototype_lyon::prelude::{Fill, Path, Stroke};

pub struct ScreenShotPlugin;

impl Plugin for ScreenShotPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ScreenshotEvent>()
            .add_event::<SvgScreenshotEvent>()
            .add_systems(Update, (screenshot_on_event, save_svg_file));
    }
}

#[derive(Event)]
pub struct ScreenshotEvent {
    pub path: String,
}

#[derive(Event)]
pub struct SvgScreenshotEvent {
    pub file_path: String,
}

fn screenshot_on_event(
    mut save_events: EventReader<ScreenshotEvent>,
    mut send_svg_events: EventWriter<SvgScreenshotEvent>,
    mut info_state: ResMut<Info>,
    main_window: Query<Entity, With<PrimaryWindow>>,
    mut screenshot_manager: ResMut<ScreenshotManager>,
    mut counter: Local<u32>,
) {
    for ScreenshotEvent { path } in save_events.iter() {
        // if there is no extension, add a
        if path.ends_with("svg") {
            info_state.notify("Writing SVG...");
            send_svg_events.send(SvgScreenshotEvent {
                file_path: path.clone(),
            });
            continue;
        }
        let suffix = if path.split('.').count() >= 2 {
            ""
        } else {
            ".png"
        };
        info_state.notify("Writing rastered image...");
        let path = format!("{path}{suffix}");
        *counter += 1;
        if let Err(e) = screenshot_manager.save_screenshot_to_disk(main_window.single(), path) {
            error!("Format not supported, try PNG, JPEG, BMP or TGA: {e}")
        }
    }
}

/// Write image to SVG.
fn save_svg_file(
    mut save_events: EventReader<SvgScreenshotEvent>,
    mut info_state: ResMut<Info>,
    map_dims: Res<MapDimensions>,
    path_query: Query<(
        &Path,
        Option<&Fill>,
        Option<&Stroke>,
        &Transform,
        &Visibility,
    )>,
    text_query: Query<(&Text, &Transform, &Visibility)>,
) {
    for SvgScreenshotEvent { file_path } in save_events.iter() {
        let fonts_dir = std::path::Path::new("./assets/fonts");
        // reflect the whole graph on both axes; the reverse step from reading from escher
        let mut writer =
            roarsvg::LyonWriter::new().with_transform(roarsvg::SvgTransform::from_scale(1.0, -1.0));
        for (path, fill, stroke, trans, vis) in &path_query {
            if Visibility::Hidden == vis {
                continue;
            }
            let (_, angle) = trans.rotation.to_axis_angle();
            // not super sure why this angle has changed sign, in histograms it is positive
            // maybe something with the scale being negative in one of the cases
            let inv_angle = match (fill, stroke) {
                (Some(_), Some(_)) => -1.0,
                _ => 1.0,
            };
            // apply its rotation and then the translation to the x center
            let svg_trans = roarsvg::SvgTransform::from_scale(trans.scale.x, trans.scale.y)
                .post_rotate((inv_angle * angle).to_degrees())
                .post_translate(trans.translation.x + map_dims.x, trans.translation.y);
            writer
                .push(
                    &path.0,
                    fill.map(|fill| {
                        let fill_color: [u8; 4] = fill.color.as_rgba_u8();
                        roarsvg::fill(
                            roarsvg::Color::new_rgb(fill_color[0], fill_color[1], fill_color[2]),
                            fill.color.a(),
                        )
                    }),
                    stroke.map(|stroke| {
                        let st_color: [u8; 4] = stroke.color.as_rgba_u8();
                        roarsvg::stroke(
                            roarsvg::Color::new_rgb(st_color[0], st_color[1], st_color[2]),
                            stroke.color.a(),
                            stroke.options.line_width,
                        )
                    }),
                    Some(svg_trans),
                )
                .unwrap_or_else(|_| info!("Writing error!"));
        }
        let mut writer = writer.add_fonts_dir(&fonts_dir);
        for (text, transform, vis) in &text_query {
            if Visibility::Hidden == vis {
                continue;
            }
            let paragraph = text
                .sections
                .iter()
                .map(|ts| &ts.value)
                .fold(String::from(""), |acc, x| acc + x.as_str());
            if paragraph.is_empty() {
                continue;
            }
            let Some((font_size, _font, color)) = text
                .sections
                .iter()
                .map(|tx| (tx.style.font_size, &tx.style.font, tx.style.color))
                .next()
            else {
                continue;
            };
            let fill: [u8; 4] = color.as_rgba_u8();
            writer
                .push_text(
                    paragraph,
                    vec![String::from("Fira Sans"), String::from("Bold")],
                    font_size,
                    roarsvg::SvgTransform::from_translate(
                        transform.translation.x + map_dims.x,
                        transform.translation.y,
                    )
                    // text rotation is actually correct, but the rest is wrong
                    // so we have to undo the global reflection
                    .pre_scale(1.0, -1.0),
                    Some(roarsvg::fill(
                        roarsvg::Color::new_rgb(fill[0], fill[1], fill[2]),
                        color.a(),
                    )),
                    None,
                )
                .unwrap_or_else(|_| info!("Writing error!"));
        }
        match writer.write(file_path) {
            Ok(_) => info_state.notify("SVG written"),
            Err(_) => info_state.notify("Error writing SVG!"),
        }
    }
}
