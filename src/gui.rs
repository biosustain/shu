//! Gui (windows and panels) to upload data and hover.

use crate::data::{Data, ReactionState};
use crate::escher::{EscherMap, MapState};
use crate::geom::{AnyTag, Xaxis};
use crate::info::Info;
use crate::screenshot::ScreenshotEvent;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::egui::color_picker::{color_edit_button_rgba, Alpha};
use bevy_egui::egui::epaint::Rgba;
use bevy_egui::egui::Hyperlink;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiSettings};
use chrono::offset::Utc;
use itertools::Itertools;
use std::collections::HashMap;

pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        let building = app
            .add_plugins(EguiPlugin)
            .insert_resource(UiState::default())
            .insert_resource(AxisMode::Hide)
            .insert_resource(ActiveData::default())
            .add_event::<SaveEvent>()
            .add_systems(Update, ui_settings)
            .add_systems(Update, scale_ui);

        // file drop and file system does not work in WASM
        #[cfg(not(target_arch = "wasm32"))]
        building.add_systems(Update, (file_drop, save_file));

        #[cfg(target_arch = "wasm32")]
        building.add_systems(Update, (listen_js_escher, listen_js_data, listen_js_info));
    }
}

#[derive(Resource)]
pub enum AxisMode {
    Show,
    Hide,
}

impl AxisMode {
    pub fn toggle(&mut self) {
        match self {
            AxisMode::Show => *self = AxisMode::Hide,
            AxisMode::Hide => *self = AxisMode::Show,
        }
    }
}

/// Retrieve a mutable reference to the color or insert
/// * a random color with the alpha that is already in the map at the empty string; or
/// * the color at the empty string (random = false).
pub fn or_color<'m>(key: &str, map: &'m mut HashMap<String, Rgba>, random: bool) -> &'m mut Rgba {
    let mut color_def = map[""];
    if random {
        map.entry(key.to_string())
            .or_insert(Rgba::from_rgba_premultiplied(
                fastrand::f32(),
                fastrand::f32(),
                fastrand::f32(),
                color_def.a(),
            ))
    } else {
        if map.contains_key(key) {
            color_def = map[key];
            map.values_mut().for_each(|v| {
                *v = color_def;
            });
        }
        map.entry(key.to_string()).or_insert(color_def)
    }
}

/// Global appeareance settings.
#[derive(Resource)]
pub struct UiState {
    pub min_reaction: f32,
    pub max_reaction: f32,
    pub zero_white: bool,
    pub min_reaction_color: Rgba,
    pub max_reaction_color: Rgba,
    pub min_metabolite: f32,
    pub max_metabolite: f32,
    pub min_metabolite_color: Rgba,
    pub max_metabolite_color: Rgba,
    pub max_left: f32,
    pub max_right: f32,
    pub max_top: f32,
    pub color_left: HashMap<String, Rgba>,
    pub color_right: HashMap<String, Rgba>,
    pub color_top: HashMap<String, Rgba>,
    pub condition: String,
    pub conditions: Vec<String>,
    pub save_path: String,
    pub map_path: String,
    pub data_path: String,
    pub screen_path: String,
    pub hide: bool,
    // since this type and field are private, Self has to be initialized
    // with Default::default(), ensuring that the fallbacks for colors (empty string) are set.
    _init: Init,
}

struct Init;

impl Default for UiState {
    fn default() -> Self {
        Self {
            min_reaction_color: Rgba::from_srgba_unmultiplied(178, 74, 74, 255),
            max_reaction_color: Rgba::from_srgba_unmultiplied(64, 169, 127, 255),
            min_metabolite_color: Rgba::from_srgba_unmultiplied(222, 208, 167, 255),
            max_metabolite_color: Rgba::from_srgba_unmultiplied(189, 143, 120, 255),
            zero_white: false,
            min_reaction: 20.,
            max_reaction: 60.,
            min_metabolite: 15.,
            max_metabolite: 50.,
            max_left: 100.,
            max_right: 100.,
            max_top: 100.,
            color_left: {
                let mut color = HashMap::new();
                color.insert(
                    String::from(""),
                    Rgba::from_srgba_unmultiplied(218, 150, 135, 190),
                );
                color
            },
            color_right: {
                let mut color = HashMap::new();
                color.insert(
                    String::from(""),
                    Rgba::from_srgba_unmultiplied(125, 206, 96, 190),
                );
                color
            },
            color_top: {
                let mut color = HashMap::new();
                color.insert(
                    String::from(""),
                    Rgba::from_srgba_unmultiplied(161, 134, 216, 190),
                );
                color
            },
            condition: String::from(""),
            conditions: vec![String::from("")],
            save_path: format!("this_map-{}.json", Utc::now().format("%T-%Y")),
            screen_path: format!("screenshot-{}.svg", Utc::now().format("%T-%Y")),
            map_path: String::from("my_map.json"),
            data_path: String::from("my_data.metabolism.json"),
            hide: false,
            _init: Init,
        }
    }
}

impl UiState {
    fn get_geom_params_mut(&mut self, extreme: &str, geom: &str) -> (&mut Rgba, &mut f32) {
        match (extreme, geom) {
            ("min", "Reaction") => (&mut self.min_reaction_color, &mut self.min_reaction),
            ("max", "Reaction") => (&mut self.max_reaction_color, &mut self.max_reaction),
            ("min", "Metabolite") => (&mut self.min_metabolite_color, &mut self.min_metabolite),
            ("max", "Metabolite") => (&mut self.max_metabolite_color, &mut self.max_metabolite),
            ("left", _) => (
                or_color(geom, &mut self.color_left, true),
                &mut self.max_left,
            ),
            ("right", _) => (
                or_color(geom, &mut self.color_right, true),
                &mut self.max_right,
            ),
            ("top", _) => (or_color(geom, &mut self.color_top, true), &mut self.max_top),
            _ => panic!("Unknown side"),
        }
    }

    fn get_mut_paths(&mut self, label: &str) -> &mut String {
        match label {
            "Map" => &mut self.map_path,
            "Data" => &mut self.data_path,
            _ => panic!("Unknown label"),
        }
    }
}

#[derive(Default)]
pub struct ActiveHists {
    pub left: bool,
    pub right: bool,
    pub top: bool,
}

#[derive(Default, Resource)]
/// Holds state about what data is being plotted, to then only show relevant
/// options in the Settings at [`ui_settings`].
pub struct ActiveData {
    pub arrow: bool,
    pub circle: bool,
    pub histogram: ActiveHists,
}

impl ActiveData {
    fn get(&self, key: &str) -> bool {
        match key {
            "Reaction" => self.arrow,
            "Metabolite" => self.circle,
            "left" => self.histogram.left,
            "right" => self.histogram.right,
            "top" => self.histogram.top,
            _ => panic!("{key} should never be an ActiveData key!"),
        }
    }

    fn any_hist(&self) -> bool {
        self.histogram.top | self.histogram.left | self.histogram.right
    }
}

#[derive(Event)]
pub struct SaveEvent(String);

/// Settings for appearance of map and plots.
/// This is managed by [`bevy_egui`] and it is separate from the rest of the GUI.
pub fn ui_settings(
    mut state: ResMut<UiState>,
    active_set: Res<ActiveData>,
    mut egui_context: EguiContexts,
    mut save_events: EventWriter<SaveEvent>,
    mut load_events: EventWriter<FileDragAndDrop>,
    mut screen_events: EventWriter<ScreenshotEvent>,
    windows: Query<(Entity, &Window), With<PrimaryWindow>>,
) {
    if state.hide {
        return;
    }
    egui::Window::new("Settings").show(egui_context.ctx_mut(), |ui| {
        ui.visuals_mut().override_text_color = Some(egui::Color32::WHITE);
        for (geom, ext) in ["Reaction", "Metabolite"]
            .into_iter()
            .cartesian_product(["min", "max"])
        {
            if !active_set.get(geom) {
                continue;
            }
            if "min" == ext {
                ui.label(format!("{geom} scale"));
            }
            let (color, value) = state.get_geom_params_mut(ext, geom);
            ui.horizontal(|ui| {
                color_edit_button_rgba(ui, color, Alpha::Opaque);
                ui.add(egui::Slider::new(value, 5.0..=90.0).text(ext));
            });
        }

        let condition = state.condition.clone();
        if (condition != "ALL") & active_set.any_hist() {
            ui.label("Histogram scale");
            for side in ["left", "right", "top"] {
                if !active_set.get(side) {
                    continue;
                }
                ui.horizontal(|ui| {
                    let (color, value) = state.get_geom_params_mut(side, &condition);
                    color_edit_button_rgba(ui, color, Alpha::BlendOrAdditive);
                    ui.add(egui::Slider::new(value, 1.0..=300.0).text(side));
                });
            }
        }

        if active_set.get("Reaction") | active_set.get("Metabolite") {
            ui.checkbox(&mut state.zero_white, "Zero as white");
        }

        if let Some(first_cond) = state.conditions.first() {
            if !((first_cond.is_empty()) & (state.conditions.len() == 1)) {
                let conditions = state.conditions.clone();
                let condition = &mut state.condition;
                egui::ComboBox::from_label("Condition")
                    .selected_text(condition.clone())
                    .show_ui(ui, |ui| {
                        for cond in conditions.iter() {
                            ui.selectable_value(condition, cond.clone(), cond.clone());
                        }
                    });
            }
        }
        // direct interactions with the file system are not supported in WASM
        // for loading, direct wasm bindings are being used.
        ui.collapsing("Export", |ui| {
            #[cfg(not(target_arch = "wasm32"))]
            ui.horizontal(|ui| {
                if ui.button("Save map").clicked() {
                    save_events.send(SaveEvent(state.save_path.clone()));
                }
                ui.text_edit_singleline(&mut state.save_path);
            });

            ui.horizontal(|ui| {
                if ui.button("Image").clicked() {
                    screen_events.send(ScreenshotEvent {
                        path: state.screen_path.clone(),
                    });
                    state.hide = true;
                }
                ui.text_edit_singleline(&mut state.screen_path);
            })
        });
        #[cfg(not(target_arch = "wasm32"))]
        ui.collapsing("Import", |ui| {
            let Ok((win, _)) = windows.get_single() else {
                return;
            };
            for label in ["Map", "Data"] {
                let path = state.get_mut_paths(label);
                ui.horizontal(|ui| {
                    if ui.button(label).clicked() {
                        // piggyback on file_drop()
                        load_events.send(FileDragAndDrop::DroppedFile {
                            window: win,
                            path_buf: path.clone().into(),
                        });
                    }
                    ui.text_edit_singleline(path);
                });
            }
        });

        ui.add(
            Hyperlink::from_label_and_url(
                "How to use?",
                "https://biosustain.github.io/shu/docs/plotting.html",
            )
            .open_in_new_tab(true),
        );
    });
}

/// Open `.metabolism.json` and `.reactions.json` files when dropped on the window.
pub fn file_drop(
    mut info_state: ResMut<Info>,
    asset_server: Res<AssetServer>,
    mut reaction_resource: ResMut<ReactionState>,
    mut escher_resource: ResMut<MapState>,
    mut events: EventReader<FileDragAndDrop>,
) {
    for event in events.read() {
        if let FileDragAndDrop::DroppedFile { path_buf, .. } = event {
            println!("Dropped file with path: {:?}", path_buf);

            let path_string = path_buf.to_str().unwrap().to_string();
            if path_buf.to_str().unwrap().ends_with("metabolism.json") {
                let reaction_handle: Handle<Data> = asset_server.load(path_string);
                reaction_resource.reaction_data = Some(reaction_handle);
                reaction_resource.loaded = false;
                info_state.notify("(gui) Loading data...");
            } else {
                //an escher map
                let escher_handle: Handle<EscherMap> = asset_server.load(path_string);
                escher_resource.escher_map = escher_handle;
                escher_resource.loaded = false;
                info_state.notify("Loading map...");
            }
        }
    }
}

/// Change size of UI on +/-.
fn scale_ui(
    key_input: Res<ButtonInput<KeyCode>>,
    mut ui_scale: ResMut<UiScale>,
    mut egui_settings_query: Query<&mut EguiSettings>,
) {
    let scale = if key_input.pressed(KeyCode::ControlLeft) {
        &mut egui_settings_query.single_mut().scale_factor
    } else {
        &mut ui_scale.0
    };
    if key_input.just_pressed(KeyCode::NumpadAdd) {
        *scale *= 1.1;
    } else if key_input.just_pressed(KeyCode::NumpadSubtract) {
        *scale /= 1.1;
    }
}

/// Save map to arbitrary place, including (non-hover) hist transforms.
fn save_file(
    mut assets: ResMut<Assets<EscherMap>>,
    mut info_state: ResMut<Info>,
    state: ResMut<MapState>,
    mut save_events: EventReader<SaveEvent>,
    hist_query: Query<(&Transform, &Xaxis), Without<AnyTag>>,
) {
    for save_event in save_events.read() {
        let custom_asset = assets.get_mut(&state.escher_map);
        if custom_asset.is_none() {
            return;
        }
        let escher_map = custom_asset.unwrap();
        for (trans, axis) in hist_query.iter() {
            if let Some(reac) = escher_map.metabolism.reactions.get_mut(&axis.node_id) {
                reac.hist_position
                    .get_or_insert(HashMap::new())
                    .insert(axis.side.clone(), (*trans).into());
            }
        }
        safe_json_write(&save_event.0, escher_map).unwrap_or_else(|e| {
            warn!("Could not write the file: {}.", e);
            info_state.notify("File could not be written!\nCheck that path exists.");
        });
    }
}

fn safe_json_write<P, C>(path: P, contents: C) -> std::io::Result<()>
where
    P: AsRef<std::path::Path>,
    C: serde::Serialize,
{
    std::fs::write(path, serde_json::to_string(&contents)?)?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
/// WASM Part.
#[derive(Resource)]
pub struct ReceiverResource<T> {
    pub rx: async_std::channel::Receiver<T>,
}

#[cfg(target_arch = "wasm32")]
fn listen_js_escher(
    receiver: Res<ReceiverResource<EscherMap>>,
    mut escher_asset: ResMut<Assets<EscherMap>>,
    mut escher_resource: ResMut<MapState>,
) {
    if let Ok(escher_map) = receiver.rx.try_recv() {
        escher_resource.escher_map = escher_asset.add(escher_map);
        escher_resource.loaded = false;
    }
}

#[cfg(target_arch = "wasm32")]
fn listen_js_data(
    receiver: Res<ReceiverResource<Data>>,
    mut data_asset: ResMut<Assets<Data>>,
    mut data_resource: ResMut<ReactionState>,
) {
    if let Ok(escher_map) = receiver.rx.try_recv() {
        data_resource.reaction_data = Some(data_asset.add(escher_map));
        data_resource.loaded = false;
    }
}

#[cfg(target_arch = "wasm32")]
fn listen_js_info(receiver: Res<ReceiverResource<&'static str>>, mut info_box: ResMut<Info>) {
    if let Ok(msg) = receiver.rx.try_recv() {
        info_box.notify(msg);
    }
}
