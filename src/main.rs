#![allow(clippy::type_complexity)]

use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::prelude::*;
use bevy::winit::WinitSettings;
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_prototype_lyon::prelude::*;
use serde::Deserialize;

mod aesthetics;
mod data;
mod escher;
mod funcplot;
mod geom;
mod gui;
mod legend;
#[cfg(test)]
mod tests;

use escher::{EscherMap, EscherPlugin, MapState};

/// Data sent from callback through the channel.
#[derive(Deserialize, Debug, Default, Clone, PartialEq)]
pub struct Example {
    pub field1: [f32; 4],
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WinitSettings::desktop_app())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "shu".to_string(),
                ..default()
            },
            ..default()
        }))
        .add_plugin(PanCamPlugin::default())
        .add_plugin(ShapePlugin)
        .add_plugin(EscherPlugin)
        .add_plugin(gui::GuiPlugin)
        .add_plugin(data::DataPlugin)
        .add_startup_system(setup_system)
        .add_plugin(aesthetics::AesPlugin)
        .add_plugin(legend::LegendPlugin)
        .run();
}

#[cfg(target_arch = "wasm32")]
/// Main function with WASM additions.
/// Three main differences:
/// - Get WASM modules.
/// - Create a button that sends data through a channel.
/// - Insert a Receiver resource so that systems can listen to that.
fn main() {
    use async_std::channel::{unbounded, Receiver, Sender};
    use gui::ReceiverResource;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::{spawn_local, JsFuture};
    use web_sys::console;
    use web_sys::HtmlInputElement;

    let (map_sender, map_receiver): (Sender<EscherMap>, Receiver<EscherMap>) = unbounded();
    let (data_sender, data_receiver): (Sender<data::Data>, Receiver<data::Data>) = unbounded();

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
                    console::warn_1(&"Provided file does not have right shape".into())
                }
            }
        })
    }) as Box<dyn FnMut(_)>);
    let data_closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
        let s = data_sender.clone();
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
                    console::warn_1(&"Provided file does not have right shape".into())
                }
            }
        })
    }) as Box<dyn FnMut(_)>);
    console::log_1(&"closure setup done!".into());
    target_map.set_onchange(Some(map_closure.as_ref().unchecked_ref()));
    target_data.set_onchange(Some(data_closure.as_ref().unchecked_ref()));

    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WinitSettings::desktop_app())
        .insert_resource(ReceiverResource { rx: map_receiver })
        .insert_resource(ReceiverResource { rx: data_receiver })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "shu".to_string(),
                fit_canvas_to_parent: true,
                canvas: Some("#bevy".to_string()),
                ..default()
            },
            ..default()
        }))
        .add_plugin(PanCamPlugin::default())
        .add_plugin(ShapePlugin)
        .add_plugin(EscherPlugin)
        .add_plugin(gui::GuiPlugin)
        .add_plugin(data::DataPlugin)
        .add_startup_system(setup_system)
        .add_plugin(aesthetics::AesPlugin)
        .add_plugin(legend::LegendPlugin)
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
        reac_loaded: false,
        met_loaded: false,
    });

    commands
        .spawn(Camera2dBundle {
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::Custom(Color::rgb(1., 1., 1.)),
            },
            ..Default::default()
        })
        .insert(PanCam {
            grab_buttons: vec![MouseButton::Left], // which buttons should drag the camera
            enabled: true, // when false, controls are disabled. See toggle example.
            zoom_to_cursor: true, // whether to zoom towards the mouse or the center of the screen
            min_scale: 1., // prevent the camera from zooming too far in
            max_scale: Some(40.), // prevent the camera from zooming too far out
            ..Default::default()
        });
}
