//! Gui (windows and panels) to upload data and hover.

use crate::data::{MetaboliteData, ReactionData, ReactionState};
use crate::escher::{EscherMap, Hover, MapState};
use crate::geom::{AnyTag, HistTag};
use bevy::prelude::*;
use bevy_egui::egui::color_picker::{color_edit_button_hsva, Alpha};
use bevy_egui::egui::epaint::color::Hsva;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use std::collections::HashMap;

pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin)
            .insert_resource(UiState::default())
            .add_system(ui_settings)
            .add_system(show_hover)
            .add_system(follow_mouse_on_drag)
            .add_system(follow_mouse_on_rotate)
            .add_system(mouse_click_system)
            .add_system(file_drop)
            .add_system(save_file);
    }
}

/// Global appeareance settings.
#[derive(Resource)]
pub struct UiState {
    pub min_reaction: f32,
    pub max_reaction: f32,
    pub min_reaction_color: Hsva,
    pub max_reaction_color: Hsva,
    pub min_metabolite: f32,
    pub max_metabolite: f32,
    pub min_metabolite_color: Hsva,
    pub max_metabolite_color: Hsva,
    pub max_left: f32,
    pub max_right: f32,
    pub max_top: f32,
    pub condition: String,
    pub conditions: Vec<String>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            min_reaction_color: Hsva::new(0.58, 0.2, 0.5, 1.),
            max_reaction_color: Hsva::new(0.58, 0.3, 0.85, 1.),
            min_metabolite_color: Hsva::new(0.28, 0.2, 0.5, 1.),
            max_metabolite_color: Hsva::new(0.28, 0.3, 0.85, 1.),
            min_reaction: 20.,
            max_reaction: 60.,
            min_metabolite: 20.,
            max_metabolite: 60.,
            max_left: 100.,
            max_right: 100.,
            max_top: 100.,
            condition: String::from(""),
            conditions: vec![String::from("")],
        }
    }
}

fn ui_settings(mut egui_context: ResMut<EguiContext>, mut ui_state: ResMut<UiState>) {
    egui::Window::new("Settings").show(egui_context.ctx_mut(), |ui| {
        ui.label("Reaction scale");
        ui.horizontal(|ui| {
            color_edit_button_hsva(ui, &mut ui_state.min_reaction_color, Alpha::Opaque);
            ui.add(egui::Slider::new(&mut ui_state.min_reaction, 5.0..=90.0).text("min"));
        });
        ui.horizontal(|ui| {
            color_edit_button_hsva(ui, &mut ui_state.max_reaction_color, Alpha::Opaque);
            ui.add(egui::Slider::new(&mut ui_state.max_reaction, 5.0..=90.0).text("max"));
        });
        ui.label("Metabolite scale");
        ui.horizontal(|ui| {
            color_edit_button_hsva(ui, &mut ui_state.min_metabolite_color, Alpha::Opaque);
            ui.add(egui::Slider::new(&mut ui_state.min_metabolite, 5.0..=90.0).text("min"));
        });
        ui.horizontal(|ui| {
            color_edit_button_hsva(ui, &mut ui_state.max_metabolite_color, Alpha::Opaque);
            ui.add(egui::Slider::new(&mut ui_state.max_metabolite, 5.0..=90.0).text("max"));
        });
        ui.label("Histogram scale");
        ui.add(egui::Slider::new(&mut ui_state.max_left, 1.0..=300.0).text("left"));
        ui.add(egui::Slider::new(&mut ui_state.max_right, 1.0..=300.0).text("right"));
        ui.add(egui::Slider::new(&mut ui_state.max_top, 1.0..=300.0).text("hover"));
        if let Some(first_cond) = ui_state.conditions.get(0) {
            if !((first_cond.is_empty()) & (ui_state.conditions.len() == 1)) {
                let conditions = ui_state.conditions.clone();
                let condition = &mut ui_state.condition;
                egui::ComboBox::from_label("Condition")
                    .selected_text(conditions[0].clone())
                    .show_ui(ui, |ui| {
                        for cond in conditions.iter() {
                            ui.selectable_value(condition, cond.clone(), cond.clone());
                        }
                    });
            }
        }
    });
}

fn file_drop(
    mut dnd_evr: EventReader<FileDragAndDrop>,
    asset_server: Res<AssetServer>,
    mut reaction_resource: ResMut<ReactionState>,
    mut escher_resource: ResMut<MapState>,
) {
    for ev in dnd_evr.iter() {
        if let FileDragAndDrop::DroppedFile { id, path_buf } = ev {
            println!("Dropped file with path: {:?}", path_buf);

            if id.is_primary() {
                // it was dropped over the main window
            }

            // it was dropped over our UI element
            // (our UI element is being hovered over)

            if path_buf.to_str().unwrap().ends_with("reaction.json") {
                let reaction_handle: Handle<ReactionData> =
                    asset_server.load(path_buf.to_str().unwrap());
                reaction_resource.reaction_data = Some(reaction_handle);
                reaction_resource.reac_loaded = false;
                info! {"Reactions dropped!"};
            } else if path_buf.to_str().unwrap().ends_with("metabolite.json") {
                let metabolite_handle: Handle<MetaboliteData> =
                    asset_server.load(path_buf.to_str().unwrap());
                reaction_resource.metabolite_data = Some(metabolite_handle);
                reaction_resource.met_loaded = false;
                info! {"Metabolites dropped!"};
            } else {
                //an escher map
                let escher_handle: Handle<EscherMap> =
                    asset_server.load(path_buf.to_str().unwrap());
                escher_resource.escher_map = escher_handle;
                escher_resource.loaded = false;
            }
        }
    }
}

/// Cursor to mouse position. Stolen from bevy cheatbook.
fn get_pos(win: &Window, camera: &Camera, camera_transform: &GlobalTransform) -> Option<Vec2> {
    // get the size of the window
    let window_size = Vec2::new(win.width(), win.height());
    if let Some(screen_pos) = win.cursor_position() {
        // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
        let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;
        // matrix for undoing the projection and camera transform
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();
        // use it to convert ndc to world-space coordinates
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
        // reduce it to a 2D value
        Some(world_pos.truncate())
    } else {
        None
    }
}

/// Show hovered data on cursor enter.
fn show_hover(
    ui_state: Res<UiState>,
    windows: Res<Windows>,
    hover_query: Query<(&Transform, &Hover)>,
    mut popup_query: Query<(&mut Visibility, &AnyTag, &HistTag)>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    let (camera, camera_transform) = q_camera.single();
    let win = windows.get_primary().expect("no primary window");
    if let Some(world_pos) = get_pos(win, camera, camera_transform) {
        for (trans, hover) in hover_query.iter() {
            if (world_pos - Vec2::new(trans.translation.x, trans.translation.y)).length_squared()
                < 5000.
            {
                for (mut vis, tag, hist) in popup_query.iter_mut() {
                    let cond_if = hist
                        .condition
                        .as_ref()
                        .map(|c| c == &ui_state.condition)
                        .unwrap_or(true);
                    if (hover.node_id == tag.id) & cond_if {
                        *vis = Visibility::VISIBLE;
                    }
                }
            } else {
                for (mut vis, tag, hist) in popup_query.iter_mut() {
                    let cond_if = hist
                        .condition
                        .as_ref()
                        .map(|c| c != &ui_state.condition)
                        .unwrap_or(false);
                    if (hover.node_id == tag.id) || cond_if {
                        *vis = Visibility::INVISIBLE;
                    }
                }
            }
        }
    }
}

/// Register an histogram as being dragged by center or right button.
fn mouse_click_system(
    windows: Res<Windows>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut drag_query: Query<(&Transform, &mut HistTag), Without<AnyTag>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    if mouse_button_input.just_pressed(MouseButton::Middle) {
        for (trans, mut hist) in drag_query.iter_mut() {
            let (camera, camera_transform) = q_camera.single();
            let win = windows.get_primary().expect("no primary window");
            if let Some(world_pos) = get_pos(win, camera, camera_transform) {
                if (world_pos - Vec2::new(trans.translation.x, trans.translation.y))
                    .length_squared()
                    < 5000.
                {
                    hist.dragged = true;
                }
            }
        }
    }

    if mouse_button_input.just_released(MouseButton::Middle) {
        for (_, mut hist) in drag_query.iter_mut() {
            hist.dragged = false;
        }
    }
    if mouse_button_input.just_pressed(MouseButton::Right) {
        for (trans, mut hist) in drag_query.iter_mut() {
            let (camera, camera_transform) = q_camera.single();
            let win = windows.get_primary().expect("no primary window");
            if let Some(world_pos) = get_pos(win, camera, camera_transform) {
                if (world_pos - Vec2::new(trans.translation.x, trans.translation.y))
                    .length_squared()
                    < 5000.
                {
                    hist.rotating = true;
                }
            }
        }
    }

    if mouse_button_input.just_released(MouseButton::Right) {
        for (_, mut hist) in drag_query.iter_mut() {
            hist.rotating = false;
        }
    }
}

/// Move the center-dragged histograms.
fn follow_mouse_on_drag(
    windows: Res<Windows>,
    mut drag_query: Query<(&mut Transform, &HistTag)>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    for (mut trans, hist) in drag_query.iter_mut() {
        if hist.dragged {
            let (camera, camera_transform) = q_camera.single();
            let win = windows.get_primary().expect("no primary window");
            if let Some(world_pos) = get_pos(win, camera, camera_transform) {
                trans.translation = Vec3::new(world_pos.x, world_pos.y, trans.translation.z);
            }
        }
    }
}

/// Rotate the right-dragged histograms.
fn follow_mouse_on_rotate(
    mut drag_query: Query<(&mut Transform, &HistTag)>,
    mut mouse_motion_events: EventReader<bevy::input::mouse::MouseMotion>,
) {
    for ev in mouse_motion_events.iter() {
        for (mut trans, hist) in drag_query.iter_mut() {
            let pos = trans.translation;
            if hist.rotating {
                trans.rotate_around(pos, Quat::from_axis_angle(Vec3::Z, -ev.delta.y * 0.05));
            }
        }
    }
}

/// Save map to arbitrary place, including (non-hover) hist transforms.
fn save_file(
    mut assets: ResMut<Assets<EscherMap>>,
    state: ResMut<MapState>,
    keyboard_input: Res<Input<KeyCode>>,
    hist_query: Query<(&Transform, &HistTag), Without<AnyTag>>,
) {
    if keyboard_input.just_released(KeyCode::S) {
        let custom_asset = assets.get_mut(&state.escher_map);
        if custom_asset.is_none() {
            return;
        }
        let escher_map = custom_asset.unwrap();
        for (trans, hist) in hist_query.iter() {
            if let Some(reac) = escher_map.metabolism.reactions.get_mut(&hist.node_id) {
                reac.hist_position
                    .get_or_insert(HashMap::new())
                    .insert(hist.side.clone(), (*trans).into());
            }
        }
        std::fs::write(
            "/tmp/random_shu_map.json",
            serde_json::to_string(escher_map).expect("Serializing the map failed!"),
        )
        .expect("Saving the model failed!");
    }
}
