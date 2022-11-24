//! Input data logic.

use std::collections::HashSet;

use crate::aesthetics;
use crate::escher::EscherMap;
use crate::geom;
use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::utils::BoxedFuture;
use itertools::Itertools;
use serde::Deserialize;

pub struct DataPlugin;

impl Plugin for DataPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<EscherMap>()
            .add_asset::<ReactionData>()
            .add_asset::<MetaboliteData>()
            .add_asset_loader(CustomAssetLoader::<EscherMap>::new(vec!["json"]))
            .add_asset_loader(CustomAssetLoader::<ReactionData>::new(vec![
                "reaction.json",
            ]))
            .add_asset_loader(CustomAssetLoader::<MetaboliteData>::new(vec![
                "metabolite.json",
            ]))
            .add_system(load_reaction_data)
            .add_system(load_metabolite_data);
    }
}

#[derive(Default)]
pub struct CustomAssetLoader<A> {
    extensions: Vec<&'static str>,
    _mark: std::marker::PhantomData<A>,
}

impl<A> AssetLoader for CustomAssetLoader<A>
where
    for<'de> A: serde::Deserialize<'de> + bevy::asset::Asset,
{
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let custom_asset = serde_json::from_slice::<A>(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(custom_asset));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &self.extensions
    }
}

impl<A> CustomAssetLoader<A> {
    fn new(extensions: Vec<&'static str>) -> Self {
        Self {
            extensions,
            _mark: std::marker::PhantomData::<A>,
        }
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
    /// Numeric values to plot as KDE.
    y: Option<Vec<Vec<f32>>>,
    /// Numeric values to plot as KDE.
    left_y: Option<Vec<Vec<f32>>>,
    /// Numeric values to plot on a hovered popup.
    hover_y: Option<Vec<Vec<f32>>>,
    /// Categorical values to be associated with conditions.
    conditions: Option<Vec<String>>,
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
    /// Numeric values to plot as histogram on hover.
    y: Option<Vec<Vec<f32>>>,
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
    current_hist: Query<Entity, Or<(With<geom::GeomHist>, With<geom::HistTag>)>>,
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
    let conditions = reacs.conditions.clone().unwrap_or(vec![String::from("")]);
    let cond_set = conditions.iter().unique();
    for cond in cond_set {
        let indices = if cond == "" {
            reacs
                .reactions
                .iter()
                .enumerate()
                .map(|(i, _)| i)
                .collect::<HashSet<usize>>()
        } else {
            conditions
                .iter()
                .enumerate()
                .filter(|(_, c)| c == &cond)
                .map(|(i, _)| i)
                .collect()
        };
        let identifiers = indices.iter().map(|i| &reacs.reactions[*i]);
        info!("{indices:?}");
        if let Some(color_data) = &mut reacs.colors {
            let mut color_data = indices.iter().map(|i| color_data[*i]).collect::<Vec<f32>>();
            // remove existing color geoms
            for e in current_colors.iter() {
                commands.entity(e).despawn_recursive();
            }
            commands
                .spawn(aesthetics::Aesthetics {
                    plotted: false,
                    identifiers: identifiers.clone().cloned().collect(),
                    condition: if cond == "" { None } else { Some(cond.clone()) },
                })
                .insert(aesthetics::Gcolor {})
                .insert(aesthetics::Point(std::mem::take(&mut color_data)))
                .insert(geom::GeomArrow { plotted: false });
        }
        if let Some(dist_data) = &mut reacs.y {
            let dist_data = indices
                .iter()
                .map(|i| std::mem::take(&mut dist_data[*i]))
                .collect::<Vec<Vec<f32>>>();
            // remove existing sizes geoms
            for e in current_hist.iter() {
                commands.entity(e).despawn_recursive();
            }
            commands
                .spawn(aesthetics::Aesthetics {
                    plotted: false,
                    identifiers: identifiers.clone().cloned().collect(),
                    condition: if cond == "" { None } else { Some(cond.clone()) },
                })
                .insert(aesthetics::Gy {})
                .insert(aesthetics::Distribution(dist_data))
                .insert(geom::GeomHist::right());
        }
        if let Some(dist_data) = &mut reacs.left_y {
            let dist_data = indices
                .iter()
                .map(|i| std::mem::take(&mut dist_data[*i]))
                .collect::<Vec<Vec<f32>>>();
            // remove existing sizes geoms
            for e in current_hist.iter() {
                commands.entity(e).despawn_recursive();
            }
            commands
                .spawn(aesthetics::Aesthetics {
                    plotted: false,
                    identifiers: identifiers.clone().cloned().collect(),
                    condition: if cond == "" { None } else { Some(cond.clone()) },
                })
                .insert(aesthetics::Gy {})
                .insert(aesthetics::Distribution(dist_data))
                .insert(geom::GeomHist::left());
        }
        if let Some(size_data) = &mut reacs.sizes {
            let mut size_data = indices.iter().map(|i| size_data[*i]).collect::<Vec<f32>>();
            // remove existing sizes geoms
            for e in current_sizes.iter() {
                commands.entity(e).despawn_recursive();
            }
            commands
                .spawn(aesthetics::Aesthetics {
                    plotted: false,
                    identifiers: identifiers.clone().cloned().collect(),
                    condition: if cond == "" { None } else { Some(cond.clone()) },
                })
                .insert(aesthetics::Gsize {})
                .insert(aesthetics::Point(std::mem::take(&mut size_data)))
                .insert(geom::GeomArrow { plotted: false });
        }
        if let Some(hover_data) = &mut reacs.hover_y {
            info!("{hover_data:?}");
            let hover_data = indices
                .iter()
                .map(|i| std::mem::take(&mut hover_data[*i]))
                .collect::<Vec<Vec<f32>>>();
            // remove existing sizes geoms
            for e in current_sizes.iter() {
                commands.entity(e).despawn_recursive();
            }
            commands
                .spawn(aesthetics::Aesthetics {
                    plotted: false,
                    identifiers: identifiers.cloned().collect(),
                    condition: if cond == "" { None } else { Some(cond.clone()) },
                })
                .insert(aesthetics::Gy {})
                .insert(aesthetics::Distribution(hover_data))
                .insert(geom::PopUp {})
                .insert(geom::GeomHist::up());
        }
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
                condition: None,
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
                condition: None,
            })
            .insert(aesthetics::Gsize {})
            .insert(aesthetics::Point(std::mem::take(size_data)))
            .insert(geom::GeomMetabolite { plotted: false });
    }
    if let Some(hover_data) = &mut reacs.y {
        // remove existing sizes geoms
        for e in current_sizes.iter() {
            commands.entity(e).despawn_recursive();
        }
        commands
            .spawn(aesthetics::Aesthetics {
                plotted: false,
                identifiers: reacs.metabolites.clone(),
                condition: None,
            })
            .insert(aesthetics::Gy {})
            .insert(aesthetics::Distribution(std::mem::take(hover_data)))
            .insert(geom::PopUp {})
            .insert(geom::GeomHist::up());
    }
    state.met_loaded = true;
}
