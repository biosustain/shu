//! Gui (windows and panels) to upload data and hover.

use crate::data::{MetaboliteData, ReactionData, ReactionState};
use crate::escher::{EscherMap, MapState};
use bevy::prelude::*;

pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(file_drop);
    }
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
