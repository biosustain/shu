use crate::escher::MapDimensions;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::{Fill, Path, Stroke};

pub struct ScreenShotPlugin;

impl Plugin for ScreenShotPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ScreenshotEvent>()
            .add_event::<SvgScreenshotEvent>()
            .add_plugins(ImageExportPlugin::default())
            .add_systems(Update, (follow_camera, repeat_screen_event, save_svg_file));
    }
    fn cleanup(&self, app: &mut App) {
        app.get_added_plugins::<ImageExportPlugin>()
            .iter_mut()
            .for_each(|m| m.threads.finish())
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

#[derive(Component)]
pub struct ScreenShotCamera;

#[derive(Component)]
pub struct ExportPreview;

/// Repeater system that lives in appworld and mutates [`ImageExportSettings`],
/// which is later moved into the render world with the received path.
///
/// This system emulates a sender since we cannot transmit a [`Event`] directly from
/// app to render worlds.
fn repeat_screen_event(
    time: Res<Time>,
    mut save_events: EventReader<ScreenshotEvent>,
    mut send_svg_events: EventWriter<SvgScreenshotEvent>,
    mut export_repeater: Query<&mut ImageExportSettings>,
    mut info_state: ResMut<Info>,
) {
    for mut writer in export_repeater.iter_mut() {
        for ScreenshotEvent { path } in save_events.iter() {
            info_state.notify("Writing to {path}");
            if path.ends_with("svg") {
                send_svg_events.send(SvgScreenshotEvent {
                    file_path: path.clone(),
                });
            } else {
                writer.path = Some(path.clone());
                writer.timer.reset();
            }
        }
        if writer.timer.tick(time.delta()).just_finished() {
            writer.path = None;
        }
    }
}

/// Write image to SVG.
fn save_svg_file(
    mut save_events: EventReader<SvgScreenshotEvent>,
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
        writer
            .write(file_path)
            .unwrap_or_else(|_| info!("Writing error!"));
    }
}

fn follow_camera(
    q_main: Query<
        (&OrthographicProjection, &Transform),
        (With<MainCamera>, Without<ScreenShotCamera>),
    >,
    mut q_screen: Query<(&mut OrthographicProjection, &mut Transform), With<ScreenShotCamera>>,
) {
    let Ok(main_camera) = q_main.get_single() else {
        return;
    };
    let Ok(ref mut screen_camera) = q_screen.get_single_mut() else {
        return;
    };
    *screen_camera.0 = main_camera.0.clone();
    *screen_camera.1 = *main_camera.1;
}

use bevy::{
    ecs::{
        query::QueryItem,
        system::{lifetimeless::SRes, SystemParamItem},
    },
    reflect::TypeUuid,
    render::{
        camera::CameraUpdateSystem,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        main_graph::node::CAMERA_DRIVER,
        render_asset::{PrepareAssetError, RenderAsset, RenderAssetPlugin, RenderAssets},
        render_graph::{Node, NodeRunError, RenderGraph, RenderGraphContext},
        render_resource::{
            Buffer, BufferDescriptor, BufferUsages, Extent3d, ImageCopyBuffer, ImageDataLayout,
            MapMode,
        },
        renderer::{RenderContext, RenderDevice},
        Render, RenderApp, RenderSet,
    },
};

use bytemuck::AnyBitPattern;
use futures::channel::oneshot;
use image::{
    error::UnsupportedErrorKind, EncodableLayout, ImageBuffer, ImageError, Pixel,
    PixelWithColorType, Rgba,
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use wgpu::Maintain;

use crate::{gui::MainCamera, info::Info};

pub const NODE_NAME: &str = "image_export";

#[derive(Clone, TypeUuid, Default, Reflect)]
#[uuid = "d619b2f8-58cf-42f6-b7da-028c0595f7aa"]
pub struct ImageExportSource(pub Handle<Image>);

impl From<Handle<Image>> for ImageExportSource {
    fn from(value: Handle<Image>) -> Self {
        Self(value)
    }
}

#[derive(Component, Clone)]
pub struct ImageExportSettings {
    /// The path that image files will be saved to.
    pub path: Option<String>,
    /// The waiting time before the path is set to None after writing it.
    pub timer: Timer,
}

pub struct GpuImageExportSource {
    pub buffer: Buffer,
    pub source_handle: Handle<Image>,
    pub source_size: Extent3d,
    pub bytes_per_row: u32,
    pub padded_bytes_per_row: u32,
}

impl RenderAsset for ImageExportSource {
    type ExtractedAsset = Self;
    type PreparedAsset = GpuImageExportSource;
    type Param = (SRes<RenderDevice>, SRes<RenderAssets<Image>>);

    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    fn prepare_asset(
        extracted_asset: Self::ExtractedAsset,
        (device, images): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
        let gpu_image = images.get(&extracted_asset.0).unwrap();

        let size = gpu_image.texture.size();
        let format = &gpu_image.texture_format;
        let bytes_per_row =
            (size.width / format.block_dimensions().0) * format.block_size(None).unwrap();
        let padded_bytes_per_row =
            RenderDevice::align_copy_bytes_per_row(bytes_per_row as usize) as u32;

        let source_size = gpu_image.texture.size();

        Ok(GpuImageExportSource {
            buffer: device.create_buffer(&BufferDescriptor {
                label: Some("Image Export Buffer"),
                size: (source_size.height * padded_bytes_per_row) as u64,
                usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
                mapped_at_creation: false,
            }),
            source_handle: extracted_asset.0.clone(),
            source_size,
            bytes_per_row,
            padded_bytes_per_row,
        })
    }
}

#[derive(Component, Clone)]
pub struct ImageExportStartFrame(u64);

impl Default for ImageExportSettings {
    fn default() -> Self {
        Self {
            path: None,
            timer: Timer::from_seconds(0.1, TimerMode::Once),
        }
    }
}

impl ExtractComponent for ImageExportSettings {
    type Query = (
        &'static Self,
        &'static Handle<ImageExportSource>,
        &'static ImageExportStartFrame,
    );
    type Filter = ();
    type Out = (Self, Handle<ImageExportSource>, ImageExportStartFrame);

    fn extract_component(
        (settings, source_handle, start_frame): QueryItem<'_, Self::Query>,
    ) -> Option<Self::Out> {
        Some((
            settings.clone(),
            source_handle.clone_weak(),
            start_frame.clone(),
        ))
    }
}

fn setup_exporters(
    mut commands: Commands,
    exporters: Query<Entity, (With<ImageExportSettings>, Without<ImageExportStartFrame>)>,
    mut frame_id: Local<u64>,
) {
    *frame_id = frame_id.wrapping_add(1);
    for entity in &exporters {
        commands
            .entity(entity)
            .insert(ImageExportStartFrame(*frame_id));
    }
}

#[derive(Bundle, Default)]
pub struct ImageExportBundle {
    pub source: Handle<ImageExportSource>,
    pub settings: ImageExportSettings,
}

#[derive(Default, Clone, Resource)]
pub struct ExportThreads {
    pub count: Arc<AtomicUsize>,
}

impl ExportThreads {
    /// Blocks the main thread until all frames have been saved successfully.
    pub fn finish(&self) {
        while self.count.load(Ordering::SeqCst) > 0 {
            std::thread::sleep(std::time::Duration::from_secs_f32(0.25));
        }
    }
}

fn save_buffer_to_disk(
    export_bundles: Query<(
        &Handle<ImageExportSource>,
        &ImageExportSettings,
        &ImageExportStartFrame,
    )>,
    sources: Res<RenderAssets<ImageExportSource>>,
    render_device: Res<RenderDevice>,
    export_threads: Res<ExportThreads>,
) {
    for (source_handle, settings, _start_frame) in &export_bundles {
        if let Some(gpu_source) = sources.get(source_handle) {
            let mut image_bytes = {
                let slice = gpu_source.buffer.slice(..);

                {
                    let (mapping_tx, mapping_rx) = oneshot::channel();

                    render_device.map_buffer(&slice, MapMode::Read, move |res| {
                        mapping_tx.send(res).unwrap();
                    });

                    render_device.poll(Maintain::Wait);
                    futures_lite::future::block_on(mapping_rx).unwrap().unwrap();
                }

                slice.get_mapped_range().to_vec()
            };

            gpu_source.buffer.unmap();

            let settings = settings.clone();
            let bytes_per_row = gpu_source.bytes_per_row as usize;
            let padded_bytes_per_row = gpu_source.padded_bytes_per_row as usize;
            let source_size = gpu_source.source_size;
            let export_threads = export_threads.clone();

            export_threads.count.fetch_add(1, Ordering::SeqCst);
            let path = match settings.path {
                Some(p) => p,
                _ => return,
            };
            let suffix = if path.split('.').count() >= 2 {
                ""
            } else {
                ".png"
            };
            let path = format!("{path}{suffix}");
            std::thread::spawn(move || {
                if bytes_per_row != padded_bytes_per_row {
                    let mut unpadded_bytes =
                        Vec::<u8>::with_capacity(source_size.height as usize * bytes_per_row);

                    for padded_row in image_bytes.chunks(padded_bytes_per_row) {
                        unpadded_bytes.extend_from_slice(&padded_row[..bytes_per_row]);
                    }

                    image_bytes = unpadded_bytes;
                }

                fn save_buffer<P: Pixel + PixelWithColorType>(
                    image_bytes: &[P::Subpixel],
                    source_size: &Extent3d,
                    path: &str,
                ) where
                    P::Subpixel: AnyBitPattern,
                    [P::Subpixel]: EncodableLayout,
                {
                    match ImageBuffer::<P, _>::from_raw(
                        source_size.width,
                        source_size.height,
                        image_bytes,
                    ) {
                        Some(buffer) => {
                            if let Err(ImageError::Unsupported(err)) = buffer.save(path) {
                                if let UnsupportedErrorKind::Format(hint) = err.kind() {
                                    println!("Image format {} is not supported", hint);
                                }
                            }
                        }
                        None => {
                            println!("Failed creating image buffer for '{}'", path);
                        }
                    }
                }

                match suffix {
                    "exr" => {
                        save_buffer::<Rgba<f32>>(
                            bytemuck::cast_slice(&image_bytes),
                            &source_size,
                            path.as_str(),
                        );
                    }
                    _ => {
                        save_buffer::<Rgba<u8>>(&image_bytes, &source_size, path.as_str());
                    }
                }

                export_threads.count.fetch_sub(1, Ordering::SeqCst);
            });
        }
    }
}

/// Plugin enabling the generation of image sequences.
#[derive(Default)]
pub struct ImageExportPlugin {
    pub threads: ExportThreads,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum ImageExportSystems {
    SetupImageExport,
    SetupImageExportFlush,
}

impl Plugin for ImageExportPlugin {
    fn build(&self, app: &mut App) {
        use ImageExportSystems::*;

        app.configure_sets(
            PostUpdate,
            (SetupImageExport, SetupImageExportFlush)
                .chain()
                .before(CameraUpdateSystem),
        )
        .register_type::<ImageExportSource>()
        .add_asset::<ImageExportSource>()
        .register_asset_reflect::<ImageExportSource>()
        .add_plugins((
            RenderAssetPlugin::<ImageExportSource>::default(),
            ExtractComponentPlugin::<ImageExportSettings>::default(),
        ))
        .add_systems(
            PostUpdate,
            (
                setup_exporters.in_set(SetupImageExport),
                apply_deferred.in_set(SetupImageExportFlush),
            ),
        );

        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .insert_resource(self.threads.clone())
            .add_systems(
                Render,
                save_buffer_to_disk
                    .after(RenderSet::Render)
                    .before(RenderSet::Cleanup),
            );

        let mut graph = render_app.world.get_resource_mut::<RenderGraph>().unwrap();

        graph.add_node(NODE_NAME, ImageExportNode);
        graph.add_node_edge(CAMERA_DRIVER, NODE_NAME);
    }
}

// RENDER NODE IMPLEMENTATION

pub struct ImageExportNode;

impl Node for ImageExportNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        for source in world.resource::<RenderAssets<ImageExportSource>>().values() {
            if let Some(gpu_image) = world
                .resource::<RenderAssets<Image>>()
                .get(&source.source_handle)
            {
                render_context.command_encoder().copy_texture_to_buffer(
                    gpu_image.texture.as_image_copy(),
                    ImageCopyBuffer {
                        buffer: &source.buffer,
                        layout: ImageDataLayout {
                            offset: 0,
                            bytes_per_row: Some(source.padded_bytes_per_row),
                            rows_per_image: None,
                        },
                    },
                    source.source_size,
                );
            }
        }

        Ok(())
    }
}
