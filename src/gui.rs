//! Gui (windows and panels) to upload data and hover.

use crate::data::{MetaboliteData, ReactionData, ReactionState};
use crate::escher::{EscherMap, Hover, MapState};
use crate::geom::{AnyTag, HistTag};
use bevy::prelude::*;
use bevy_egui::egui::color_picker::{color_edit_button_hsva, Alpha};
use bevy_egui::egui::epaint::color::Hsva;
use bevy_egui::{egui, EguiContext, EguiPlugin};

pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin)
            .insert_resource(UiState::default())
            .add_system(ui_example)
            .add_system(show_hover)
            .add_system(file_drop);
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

fn ui_example(mut egui_context: ResMut<EguiContext>, mut ui_state: ResMut<UiState>) {
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
        let conditions = ui_state.conditions.clone();
        let condition = &mut ui_state.condition;
        egui::ComboBox::from_label("Condition")
            .selected_text(conditions[0].clone())
            .show_ui(ui, |ui| {
                for cond in conditions.iter() {
                    ui.selectable_value(condition, cond.clone(), cond.clone());
                }
            });
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

/// Show hovered data on cursor enter.
fn show_hover(
    windows: Res<Windows>,
    hover_query: Query<(&Transform, &Hover)>,
    mut popup_query: Query<(&mut Visibility, &AnyTag), With<HistTag>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    let (camera, camera_transform) = q_camera.single();
    let win = windows.get_primary().expect("no primary window");
    if let Some(screen_pos) = win.cursor_position() {
        // get the size of the window
        let window_size = Vec2::new(win.width() as f32, win.height() as f32);

        // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
        let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

        // matrix for undoing the projection and camera transform
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();

        // use it to convert ndc to world-space coordinates
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

        // reduce it to a 2D value
        let world_pos: Vec2 = world_pos.truncate();

        for (trans, hover) in hover_query.iter() {
            // info!("{} - {world_pos}", trans.translation);
            if (world_pos - Vec2::new(trans.translation.x, trans.translation.y)).length_squared()
                < 5000.
            {
                for (mut vis, tag) in popup_query.iter_mut() {
                    if hover.node_id == tag.id {
                        *vis = Visibility::VISIBLE;
                    }
                }
            } else {
                for (mut vis, tag) in popup_query.iter_mut() {
                    if hover.node_id == tag.id {
                        *vis = Visibility::INVISIBLE;
                    }
                }
            }
        }
    }
}
