//! Input data logic.

use crate::aesthetics;
use crate::geom;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use serde::Deserialize;

pub struct DataPlugin;

impl Plugin for DataPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<ReactionData>()
            .add_asset::<MetaboliteData>()
            .add_system(load_reaction_data)
            .add_system(load_metabolite_data);
    }
}

#[derive(Deserialize, TypeUuid, Default)]
#[uuid = "413be529-bfeb-41a3-8db0-4b8b382a2c46"]
pub struct ReactionData {
    /// Vector of reactions' identifiers
    reactions: Vec<String>,
    // TODO: generalize this for any Data Type and use them (from escher.rs)
    /// Numeric values to plot as reaction arrow colors.
    colors: Option<Vec<f32>>,
    /// Numeric values to plot as reaction arrow sizes.
    sizes: Option<Vec<f32>>,
}

#[derive(Deserialize, TypeUuid, Default)]
#[uuid = "423be529-cfeb-41a3-8db0-4b8b382a2c46"]
pub struct MetaboliteData {
    /// Vector of metabolites' identifiers
    metabolites: Vec<String>,
    // TODO: generalize this for any Data Type and use them (from escher.rs)
    /// Numeric values to plot as metabolite circle colors.
    colors: Option<Vec<f32>>,
    /// Numeric values to plot as metabolite circle sizes.
    sizes: Option<Vec<f32>>,
}

#[derive(Resource)]
pub struct ReactionState {
    pub reaction_data: Option<Handle<ReactionData>>,
    pub metabolite_data: Option<Handle<MetaboliteData>>,
    pub reac_loaded: bool,
    pub met_loaded: bool,
}

fn load_reaction_data(
    mut commands: Commands,
    mut state: ResMut<ReactionState>,
    mut custom_assets: ResMut<Assets<ReactionData>>,
    current_sizes: Query<Entity, (With<aesthetics::Gsize>, With<geom::GeomArrow>)>,
    current_colors: Query<Entity, (With<aesthetics::Gcolor>, With<geom::GeomArrow>)>,
) {
    let custom_asset = if let Some(reac_handle) = &mut state.reaction_data {
        custom_assets.get_mut(reac_handle)
    } else {
        return;
    };
    if state.reac_loaded || custom_asset.is_none() {
        return;
    }
    info!("Loading Reaction data!");
    let reacs = custom_asset.unwrap();
    if let Some(color_data) = &mut reacs.colors {
        // remove existing color geoms
        for e in current_colors.iter() {
            commands.entity(e).despawn_recursive();
        }
        commands
            .spawn(aesthetics::Aesthetics {
                plotted: false,
                identifiers: reacs.reactions.clone(),
            })
            .insert(aesthetics::Gcolor {})
            .insert(aesthetics::Point(std::mem::take(color_data)))
            .insert(geom::GeomArrow { plotted: false });
    }
    if let Some(size_data) = &mut reacs.sizes {
        // remove existing sizes geoms
        for e in current_sizes.iter() {
            commands.entity(e).despawn_recursive();
        }
        commands
            .spawn(aesthetics::Aesthetics {
                plotted: false,
                identifiers: reacs.reactions.clone(),
            })
            .insert(aesthetics::Gsize {})
            .insert(aesthetics::Point(std::mem::take(size_data)))
            .insert(geom::GeomArrow { plotted: false });
    }
    state.reac_loaded = true;
}

fn load_metabolite_data(
    mut commands: Commands,
    mut state: ResMut<ReactionState>,
    mut custom_assets: ResMut<Assets<MetaboliteData>>,
    current_sizes: Query<Entity, (With<aesthetics::Gsize>, With<geom::GeomMetabolite>)>,
    current_colors: Query<Entity, (With<aesthetics::Gcolor>, With<geom::GeomMetabolite>)>,
) {
    let custom_asset = if let Some(met_handle) = &mut state.metabolite_data {
        custom_assets.get_mut(met_handle)
    } else {
        return;
    };
    if state.met_loaded || custom_asset.is_none() {
        return;
    }
    info!("Loading Metabolite data!");
    let reacs = custom_asset.unwrap();
    if let Some(color_data) = &mut reacs.colors {
        // remove existing color geoms
        for e in current_colors.iter() {
            commands.entity(e).despawn_recursive();
        }
        commands
            .spawn(aesthetics::Aesthetics {
                plotted: false,
                identifiers: reacs.metabolites.clone(),
            })
            .insert(aesthetics::Gcolor {})
            .insert(aesthetics::Point(std::mem::take(color_data)))
            .insert(geom::GeomMetabolite { plotted: false });
    }
    if let Some(size_data) = &mut reacs.sizes {
        // remove existing sizes geoms
        for e in current_sizes.iter() {
            commands.entity(e).despawn_recursive();
        }
        commands
            .spawn(aesthetics::Aesthetics {
                plotted: false,
                identifiers: reacs.metabolites.clone(),
            })
            .insert(aesthetics::Gsize {})
            .insert(aesthetics::Point(std::mem::take(size_data)))
            .insert(geom::GeomMetabolite { plotted: false });
    }
    state.met_loaded = true;
}
