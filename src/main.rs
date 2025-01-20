#![allow(clippy::type_complexity, clippy::too_many_arguments)]

use bevy::prelude::*;
use bevy::winit::WinitSettings;
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_prototype_lyon::prelude::*;

mod aesthetics;
#[cfg(not(target_arch = "wasm32"))]
mod cli;
mod data;
mod escher;
mod funcplot;
mod geom;
mod gui;
mod info;
mod legend;
mod picking;
mod screenshot;
#[cfg(test)]
mod tests;

use escher::{EscherMap, EscherPlugin, MapState};

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let mut app = App::new();
    let app = app
        .insert_resource(WinitSettings::desktop_app())
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "shu".to_string(),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_linear()),
        )
        // plugins from dependencies
        .add_plugins((PanCamPlugin, ShapePlugin))
        // internal plugins
        .add_plugins(screenshot::ScreenShotPlugin)
        .add_plugins(info::InfoPlugin)
        .add_plugins(EscherPlugin)
        .add_plugins(gui::GuiPlugin)
        .add_plugins(picking::PickingPlugin)
        .add_plugins(data::DataPlugin)
        .add_systems(Startup, setup_system)
        .add_plugins(aesthetics::AesPlugin)
        .add_plugins(legend::LegendPlugin);

    let cli_args = cli::parse_args();
    if let Err(e) = cli::handle_cli_args(app, cli_args) {
        use cli::InitCliError::*;
        match e {
            InvalidPathError(error) => error!("Supplied path as arg is invalid: {error}"),
            UninitWindow => error!("Window was not initialized!"),
        }
    }

    app.run();
}

#[cfg(target_arch = "wasm32")]
/// Main function with WASM additions.
/// Three main differences:
/// - Get WASM modules.
/// - Create a button that sends data through a channel.
/// - Insert a Receiver resource so that systems can listen to that.
fn main() {
    use async_std::channel::{unbounded, Receiver, Sender};
    use bevy::asset::AssetMetaCheck;
    use gui::ReceiverResource;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::{spawn_local, JsFuture};
    use web_sys::console;
    use web_sys::HtmlInputElement;

    let (map_sender, map_receiver): (Sender<EscherMap>, Receiver<EscherMap>) = unbounded();
    let (data_sender, data_receiver): (Sender<data::Data>, Receiver<data::Data>) = unbounded();

    // I/O feedback
    // there are two senders, one for the map and one for the data
    let (info_sender, info_receiver): (Sender<&'static str>, Receiver<&'static str>) = unbounded();
    let info_log1 = info_sender.clone();

    // When building for WASM, print panics to the browser console
    console_error_panic_hook::set_once();
    let document = web_sys::window().unwrap().document().unwrap();
    // button for loading maps
    let target_map = document
        .create_element("input")
        .unwrap_throw()
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    target_map.set_type("file");
    target_map.set_name("fileb");
    target_map.set_id("fileb");
    target_map.set_class_name("fileb");
    // button for loading data
    let target_data = document
        .create_element("input")
        .unwrap_throw()
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    target_data.set_type("file");
    target_data.set_name("fileData");
    target_data.set_id("fileData");
    target_data.set_class_name("fileData");

    let body = document.body().unwrap();
    body.append_child(&target_map).unwrap();
    body.append_child(&target_data).unwrap();

    let map_closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
        let s = map_sender.clone();
        let info_log = info_log1.clone();
        spawn_local(async move {
            console::log_1(&"checking closure".into());
            if let Some(Some(file_list)) = event.target().map(|t| {
                t.dyn_ref::<HtmlInputElement>()
                    .expect("target_brows is an <input>")
                    .files()
            }) {
                let text = JsFuture::from(file_list.get(0).unwrap().text())
                    .await
                    .unwrap()
                    .as_string()
                    .unwrap();
                if let Ok(escher_map) = serde_json::from_str(&text) {
                    s.send(escher_map).await.unwrap();
                } else {
                    console::warn_1(&"Provided map does not have right shape".into());
                    info_log
                        .send("Failed loading map! Check that you JSON is correct.")
                        .await
                        .unwrap();
                }
            }
        })
    }) as Box<dyn FnMut(_)>);
    let data_closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
        let s = data_sender.clone();
        let info_log = info_sender.clone();
        spawn_local(async move {
            console::log_1(&"checking closure".into());
            if let Some(Some(file_list)) = event.target().map(|t| {
                t.dyn_ref::<HtmlInputElement>()
                    .expect("target_brows is an <input>")
                    .files()
            }) {
                let text = JsFuture::from(file_list.get(0).unwrap().text())
                    .await
                    .unwrap()
                    .as_string()
                    .unwrap();
                if let Ok(data) = serde_json::from_str(&text) {
                    s.send(data).await.unwrap();
                } else {
                    console::warn_1(&"Provided file does not have right shape".into());
                    info_log
                        .send("Failed loading data! Check that you metabolism.json is correct.")
                        .await
                        .unwrap();
                }
            }
        })
    }) as Box<dyn FnMut(_)>);
    console::log_1(&"closure setup done!".into());
    target_map.set_onchange(Some(map_closure.as_ref().unchecked_ref()));
    target_data.set_onchange(Some(data_closure.as_ref().unchecked_ref()));

    App::new()
        .insert_resource(WinitSettings::desktop_app())
        .insert_resource(ReceiverResource { rx: map_receiver })
        .insert_resource(ReceiverResource { rx: data_receiver })
        .insert_resource(ReceiverResource { rx: info_receiver })
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "shu".to_string(),
                        canvas: Some("#bevy".to_string()),
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
        )
        // plugins from dependencies
        .add_plugins((PanCamPlugin, ShapePlugin))
        // internal plugins
        .add_plugins(screenshot::ScreenShotPlugin)
        .add_plugins(info::InfoPlugin)
        .add_plugins(EscherPlugin)
        .add_plugins(gui::GuiPlugin)
        .add_plugins(picking::PickingPlugin)
        .add_plugins(data::DataPlugin)
        .add_systems(Startup, setup_system)
        .add_plugins(aesthetics::AesPlugin)
        .add_plugins(legend::LegendPlugin)
        .run();
}

fn setup_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let escher_handle: Handle<EscherMap> = asset_server.load("ecoli_core_map.json");
    commands.insert_resource(MapState {
        escher_map: escher_handle,
        loaded: false,
    });
    commands.insert_resource(data::ReactionState {
        reaction_data: None,
        loaded: false,
    });

    commands
        .spawn((
            Camera2d,
            Camera {
                clear_color: ClearColorConfig::Custom(Color::srgb(1., 1., 1.)),
                ..Default::default()
            },
            Msaa::Sample4,
        ))
        .insert(PanCam {
            grab_buttons: vec![MouseButton::Left], // which buttons should drag the camera
            enabled: true, // when false, controls are disabled. See toggle example.
            zoom_to_cursor: true, // whether to zoom towards the mouse or the center of the screen
            min_scale: 1., // prevent the camera from zooming too far in
            max_scale: 40., // prevent the camera from zooming too far out
            ..Default::default()
        });
}
