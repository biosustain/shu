//! Gui (windows and panels) to upload data and hover.

use crate::data::{MetaboliteData, ReactionData, ReactionState};
use crate::escher::{EscherMap, MapState};
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
