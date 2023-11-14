use std::path::PathBuf;

use crate::{
    escher::MapDimensions,
    funcplot::IgnoreSave,
    geom::Drag,
    gui::UiState,
    info::Info,
    legend::{Xmax, Xmin},
};
use bevy::{asset::LoadedAsset, window::PrimaryWindow};
use bevy::{prelude::*, reflect::TypeUuid};
use bevy::{reflect::TypePath, render::view::screenshot::ScreenshotManager};
use bevy_prototype_lyon::prelude::{Fill, Path, Stroke};
use image::ImageOutputFormat::Png;
use serde::Deserialize;

pub struct ScreenShotPlugin;

impl Plugin for ScreenShotPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ScreenshotEvent>()
            .add_event::<SvgScreenshotEvent>()
            .add_asset::<RawAsset>()
            .init_asset_loader::<RawAssetLoader>()
            .add_systems(Startup, setup_timer)
            .add_systems(
                Update,
                (
                    screenshot_on_event.before(crate::gui::ui_settings),
                    save_svg_file,
                ),
            );
    }
}

#[derive(Event)]
pub struct ScreenshotEvent {
    pub path: PathBuf,
}

#[derive(Event)]
pub struct SvgScreenshotEvent {
    pub file_path: String,
}

#[derive(Component, Deref, DerefMut)]
struct HideUiTimer(Timer);

fn setup_timer(mut commands: Commands) {
    commands.spawn(HideUiTimer(Timer::from_seconds(0.2, TimerMode::Once)));
}

fn screenshot_on_event(
    mut save_events: EventReader<ScreenshotEvent>,
    mut send_svg_events: EventWriter<SvgScreenshotEvent>,
    time: Res<Time>,
    mut ui_state: ResMut<UiState>,
    mut info_state: ResMut<Info>,
    mut screenshot_manager: ResMut<ScreenshotManager>,
    main_window: Query<Entity, With<PrimaryWindow>>,
    mut timer: Query<&mut HideUiTimer>,
    mut counter: Local<u32>,
) {
    let Ok(mut timer) = timer.get_single_mut() else {
        return;
    };
    if timer.tick(time.delta()).just_finished() {
        ui_state.hide = false;
    }
    for ScreenshotEvent { path } in save_events.iter() {
        timer.reset();
        let path = path.to_string_lossy();
        if path.ends_with("svg") {
            info_state.notify("Writing SVG...");
            send_svg_events.send(SvgScreenshotEvent {
                file_path: path.to_string(),
            });
            continue;
        }
        // if there is no extension, add png
        let suffix = if path.split('.').count() >= 2 {
            ""
        } else {
            ".png"
        };
        info!("Writing raster imag...");
        let path = format!("{path}{suffix}");
        *counter += 1;
        if let Err(e) = screenshot_manager.save_screenshot_to_disk(main_window.single(), path) {
            error!("Format not supported, try PNG, JPEG, BMP or TGA: {e}")
        }
    }
}

#[derive(Debug, Clone, Deserialize, TypeUuid, TypePath)]
#[uuid = "39cadc56-ab9c-4543-8640-a018b74b5152"]
pub struct RawAsset {
    pub value: Vec<u8>,
}
#[derive(Default)]
pub struct RawAssetLoader;

impl bevy::asset::AssetLoader for RawAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let raw = RawAsset {
                value: bytes.to_vec(),
            };
            load_context.set_default_asset(LoadedAsset::new(raw));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["tttx"]
    }
}

#[derive(Resource)]
/// Resource to store the two fonts used to render the map as raw `Vec<u8>`.
///
/// This is needed to pass the fonts as raw data to usvg since the bevy `Font` struct
/// does not provide a way to retrieve that data (it is a `FontArc`).
pub struct RawFontStorage {
    pub fira: Handle<RawAsset>,
    pub assis: Handle<RawAsset>,
}

/// Write image to SVG.
fn save_svg_file(
    mut save_events: EventReader<SvgScreenshotEvent>,
    mut info_state: ResMut<Info>,
    ui_scale: Res<UiScale>,
    map_dims: Res<MapDimensions>,
    // to get images and font raw data
    images: Res<Assets<Image>>,
    fonts_storage: Res<RawFontStorage>,
    raw_fonts: Res<Assets<RawAsset>>,
    path_query: Query<(
        &Path,
        Option<&Fill>,
        Option<&Stroke>,
        &Transform,
        &Visibility,
    )>,
    text_query: Query<
        (&Text, &Transform, &Visibility),
        (Without<Xmin>, Without<Xmax>, Without<IgnoreSave>),
    >,
    // legend part
    legend_query: Query<(&GlobalTransform, &Node), With<Drag>>,
    legend_node_query: Query<(Entity, &GlobalTransform, &Style, &Children)>,
    img_query: Query<(&UiImage, &Node)>,
    legend_text_query: Query<(&Text, &GlobalTransform, &Style, &Node), Without<IgnoreSave>>,
) {
    for SvgScreenshotEvent { file_path } in save_events.iter() {
        let RawAsset { value: fira } = raw_fonts.get(&fonts_storage.fira).unwrap();
        let RawAsset { value: assis } = raw_fonts.get(&fonts_storage.assis).unwrap();
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
        let writer = writer.add_fonts_source(fira);
        let mut writer = writer.add_fonts_source(assis);
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
        if let Ok((legend_trans, _legend_root)) = legend_query.get_single() {
            // legend is tricky because the reflection point is not the origin of each
            // element, all the legend itself. Thus, everything is added to a group node
            // which is then reflected.
            let mut legend_nodes = Vec::new();
            for (_parent, trans, style, children) in &legend_node_query {
                if style.display == Display::None {
                    continue;
                }
                for child in children.iter() {
                    if let Ok((img_legend, ui_node)) = img_query.get(*child) {
                        let handle = images.get_handle(&img_legend.texture);
                        let img = images.get(&handle).unwrap();
                        let Ok(img) = img.clone().try_into_dynamic() else {
                            continue;
                        };
                        let mut img_buffer = Vec::<u8>::new();
                        img.write_to(&mut std::io::Cursor::new(&mut img_buffer), Png)
                            .unwrap();
                        let trans = trans.compute_transform();
                        legend_nodes.push(
                            roarsvg::create_png_node(
                                &img_buffer,
                                roarsvg::SvgTransform::from_translate(
                                    trans.translation.x - ui_node.size().x / 2.,
                                    trans.translation.y - ui_node.size().y / 2.,
                                ),
                                ui_node.size().x,
                                ui_node.size().y,
                            )
                            .unwrap(),
                        );
                    } else if let Ok((text, child_trans, vis, ui_node)) =
                        legend_text_query.get(*child)
                    {
                        if Display::None == vis.display {
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
                        let trans = child_trans.compute_transform();
                        legend_nodes.push(
                            roarsvg::create_text_node(
                                paragraph,
                                roarsvg::SvgTransform::from_translate(
                                    // I think this has to do with padding and margins
                                    trans.translation.x - ui_node.size().x / 1.5,
                                    trans.translation.y + ui_node.size().y / 2.8,
                                ),
                                Some(roarsvg::fill(
                                    roarsvg::Color::new_rgb(fill[0], fill[1], fill[2]),
                                    color.a(),
                                )),
                                None,
                                vec![String::from("Assistant"), String::from("Regular")],
                                font_size,
                            )
                            .unwrap(),
                        );
                    }
                }
            }
            if !legend_nodes.is_empty() {
                writer
                    // undo the scaling done on the whole SVG only for the legend
                    .push_group(
                        legend_nodes,
                        roarsvg::SvgTransform::from_scale(
                            ui_scale.scale as f32,
                            -ui_scale.scale as f32,
                        )
                        .post_translate(legend_trans.translation().x, legend_trans.translation().y),
                    )
                    .unwrap();
            }
        }
        match writer.write(file_path) {
            Ok(_) => info_state.notify("SVG written"),
            Err(e) => {
                info_state.notify("Error writing SVG!");
                info!("{:?}", e);
            }
        }
    }
}
