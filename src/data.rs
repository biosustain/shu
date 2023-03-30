//! Input data logic.

use std::collections::HashSet;

use crate::aesthetics;
use crate::escher::EscherMap;
use crate::geom;
use crate::geom::{AesFilter, GeomHist, HistPlot};
use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};
use bevy::ecs::query::ReadOnlyWorldQuery;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::utils::BoxedFuture;
use itertools::Itertools;
use serde::Deserialize;

pub struct DataPlugin;

impl Plugin for DataPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<EscherMap>()
            .add_asset::<Data>()
            .add_asset_loader(CustomAssetLoader::<EscherMap>::new(vec!["json"]))
            .add_asset_loader(CustomAssetLoader::<Data>::new(vec!["metabolism.json"]))
            .add_system(load_data);
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

#[derive(Deserialize)]
#[serde(untagged)]
/// Enum to represent floats that may be NaN or Inf.
enum Number {
    Num(f32),
    Skip(String),
}

impl From<Number> for Option<f32> {
    fn from(value: Number) -> Self {
        match value {
            Number::Num(num) => Some(num),
            _ => None,
        }
    }
}

impl Number {
    fn as_ref(&self) -> Option<&f32> {
        match self {
            Number::Num(num) => Some(num),
            _ => None,
        }
    }
}

/// Metabolic data from the user that can be read from a `file.metabolism.json`.
#[derive(Deserialize, TypeUuid, Default)]
#[uuid = "413be529-bfeb-41a3-8db0-4b8b382a2c46"]
pub struct Data {
    /// Vector of reactions' identifiers
    reactions: Option<Vec<String>>,
    // TODO: generalize this for any Data Type and use them (from escher.rs)
    /// Numeric values to plot as reaction arrow colors.
    colors: Option<Vec<Number>>,
    /// Numeric values to plot as reaction arrow sizes.
    sizes: Option<Vec<Number>>,
    /// Numeric values to plot as KDE.
    y: Option<Vec<Vec<Number>>>,
    /// Numeric values to plot as KDE.
    left_y: Option<Vec<Vec<Number>>>,
    /// Numeric values to plot on a hovered popup.
    hover_y: Option<Vec<Vec<Number>>>,
    /// Numeric values to plot as KDE.
    kde_y: Option<Vec<Vec<Number>>>,
    /// Numeric values to plot as KDE.
    kde_left_y: Option<Vec<Vec<Number>>>,
    /// Numeric values to plot on a hovered popup.
    kde_hover_y: Option<Vec<Vec<Number>>>,
    /// Numeric values to plot as KDE.
    box_y: Option<Vec<Number>>,
    /// Numeric values to plot as KDE.
    box_left_y: Option<Vec<Number>>,
    /// Categorical values to be associated with conditions.
    conditions: Option<Vec<String>>,
    /// Categorical values to be associated with conditions.
    met_conditions: Option<Vec<String>>,
    /// Vector of metabolites' identifiers
    metabolites: Option<Vec<String>>,
    // TODO: generalize this for any Data Type and use them (from escher.rs)
    /// Numeric values to plot as metabolite circle colors.
    met_colors: Option<Vec<Number>>,
    /// Numeric values to plot as metabolite circle sizes.
    met_sizes: Option<Vec<Number>>,
    /// Numeric values to plot as histogram on hover.
    met_y: Option<Vec<Vec<Number>>>,
    /// Numeric values to plot as density on hover.
    kde_met_y: Option<Vec<Vec<Number>>>,
}

/// Resource that contains a [`Handle`] to user data. Modified when new datas comes in.
#[derive(Resource)]
pub struct ReactionState {
    pub reaction_data: Option<Handle<Data>>,
    pub reac_loaded: bool,
    pub met_loaded: bool,
}

struct GgPair<'a, Aes, Geom> {
    aes_component: Aes,
    geom_component: Geom,
    cond: &'a str,
    hover: bool,
    met: bool,
}

#[allow(clippy::too_many_arguments)]
fn load_data(
    mut commands: Commands,
    mut state: ResMut<ReactionState>,
    mut custom_assets: ResMut<Assets<Data>>,
    current_sizes: Query<Entity, (With<aesthetics::Gsize>, With<geom::GeomArrow>)>,
    current_colors: Query<Entity, (With<aesthetics::Gcolor>, With<geom::GeomArrow>)>,
    current_hist: Query<(Entity, &AesFilter), Or<(With<GeomHist>, With<geom::HistTag>)>>,
    current_met_sizes: Query<Entity, (With<aesthetics::Gsize>, With<geom::GeomMetabolite>)>,
    current_met_colors: Query<Entity, (With<aesthetics::Gcolor>, With<geom::GeomMetabolite>)>,
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
    let data = custom_asset.unwrap();
    let conditions = data
        .conditions
        .clone()
        .unwrap_or_else(|| vec![String::from("")]);
    let cond_set = conditions.iter().unique().collect::<HashSet<&String>>();
    if let Some(reactions) = data.reactions.as_ref() {
        for cond in cond_set.iter() {
            let indices: HashSet<usize> = if cond.is_empty() & (conditions.len() <= 1) {
                reactions
                    .iter()
                    .enumerate()
                    .map(|(i, _)| i)
                    .collect::<HashSet<usize>>()
            } else {
                conditions
                    .iter()
                    .enumerate()
                    .filter(|(i, c)| (c == cond) & (i < &reactions.len()))
                    .map(|(i, _)| i)
                    .collect()
            };
            let identifiers = indices
                .iter()
                .map(|i| reactions[*i].clone())
                .collect::<Vec<String>>();
            if let Some(ref mut point_data) = &mut data.colors {
                insert_geom_map(
                    &mut commands,
                    &indices,
                    point_data,
                    &identifiers,
                    &current_colors,
                    GgPair {
                        aes_component: aesthetics::Gcolor {},
                        geom_component: geom::GeomArrow { plotted: false },
                        cond,
                        hover: false,
                        met: false,
                    },
                );
            }

            if let Some(ref mut point_data) = &mut data.sizes {
                {
                    insert_geom_map(
                        &mut commands,
                        &indices,
                        point_data,
                        &identifiers,
                        &current_sizes,
                        GgPair {
                            aes_component: aesthetics::Gsize {},
                            geom_component: geom::GeomArrow { plotted: false },
                            cond,
                            hover: false,
                            met: false,
                        },
                    );
                };
            }
            for (i, (aes, geom_component)) in [
                (&mut data.y, GeomHist::right(HistPlot::Hist)),
                (&mut data.left_y, GeomHist::left(HistPlot::Hist)),
                (&mut data.kde_y, GeomHist::right(HistPlot::Kde)),
                (&mut data.kde_left_y, GeomHist::left(HistPlot::Kde)),
                (&mut data.hover_y, GeomHist::up(HistPlot::Hist)),
                (&mut data.kde_hover_y, GeomHist::up(HistPlot::Kde)),
            ]
            .into_iter()
            .enumerate()
            {
                if let Some(dist_data) = aes.as_mut() {
                    insert_geom_hist(
                        &mut commands,
                        dist_data,
                        &indices,
                        &identifiers,
                        &current_hist,
                        GgPair {
                            aes_component: aesthetics::Gy {},
                            geom_component,
                            cond,
                            hover: i > 3,
                            met: false,
                        },
                    );
                }
            }
            for (var, geom) in [
                (&mut data.box_y, GeomHist::right(HistPlot::BoxPoint)),
                (&mut data.box_left_y, GeomHist::left(HistPlot::BoxPoint)),
            ]
            .into_iter()
            {
                if let Some(point_data) = var {
                    let (mut data, ids): (Vec<f32>, Vec<String>) = indices
                        .iter()
                        .map(|i| &point_data[*i])
                        .zip(identifiers.iter())
                        // filter values that are NaN
                        .filter_map(|(col, id)| col.as_ref().map(|x| (*x, id.clone())))
                        .unzip();
                    // remove existing sizes geoms
                    for (e, _) in current_hist.iter().filter(|(_e, mark)| mark.pbox) {
                        commands.entity(e).despawn_recursive();
                    }
                    if data.is_empty() {
                        continue;
                    }
                    commands.spawn((
                        aesthetics::Gy {},
                        aesthetics::Point(std::mem::take(&mut data)),
                        geom,
                        AesFilter {
                            met: false,
                            pbox: true,
                        },
                        aesthetics::Aesthetics {
                            identifiers: ids,
                            condition: if cond.is_empty() {
                                None
                            } else {
                                Some(cond.to_string())
                            },
                        },
                    ));
                }
            }
        }
    }

    info!("Loading Metabolite data!");
    let conditions = data
        .met_conditions
        .clone()
        .unwrap_or_else(|| vec![String::from("")]);
    let cond_set = conditions.iter().unique().collect::<HashSet<&String>>();
    if let Some(metabolites) = data.metabolites.as_ref() {
        for cond in cond_set {
            let indices: HashSet<usize> = if cond.is_empty() & (conditions.len() == 1) {
                metabolites
                    .iter()
                    .enumerate()
                    .map(|(i, _)| i)
                    .collect::<HashSet<usize>>()
            } else {
                conditions
                    .iter()
                    .enumerate()
                    .filter(|(i, c)| (c == &cond) & (i < &metabolites.len()))
                    .map(|(i, _)| i)
                    .collect()
            };
            let identifiers = indices
                .iter()
                .map(|i| metabolites[*i].clone())
                .collect::<Vec<String>>();
            if let Some(color_data) = &mut data.met_colors {
                insert_geom_map(
                    &mut commands,
                    &indices,
                    color_data,
                    &identifiers,
                    &current_met_colors,
                    GgPair {
                        aes_component: aesthetics::Gcolor {},
                        geom_component: geom::GeomMetabolite { plotted: false },
                        cond,
                        hover: false,
                        met: false,
                    },
                );
            }
            if let Some(size_data) = &mut data.met_sizes {
                insert_geom_map(
                    &mut commands,
                    &indices,
                    size_data,
                    &identifiers,
                    &current_met_sizes,
                    GgPair {
                        aes_component: aesthetics::Gsize {},
                        geom_component: geom::GeomMetabolite { plotted: false },
                        cond,
                        hover: false,
                        met: false,
                    },
                );
            }
            for (aes, geom_component) in [
                (&mut data.met_y, GeomHist::up(HistPlot::Hist)),
                (&mut data.kde_met_y, GeomHist::up(HistPlot::Kde)),
            ]
            .into_iter()
            {
                if let Some(dist_data) = aes {
                    insert_geom_hist(
                        &mut commands,
                        dist_data,
                        &indices,
                        &identifiers,
                        &current_hist,
                        GgPair {
                            aes_component: aesthetics::Gy {},
                            geom_component,
                            cond,
                            hover: true,
                            met: true,
                        },
                    );
                }
            }
        }
    }

    state.met_loaded = true;
    state.reac_loaded = true;
}

fn insert_geom_map<F, Aes: Component, Geom: Component>(
    commands: &mut Commands,
    indices: &HashSet<usize>,
    aes_data: &mut [Number],
    identifiers: &[String],
    to_remove: &Query<Entity, F>,
    ggcomp: GgPair<Aes, Geom>,
) where
    F: ReadOnlyWorldQuery,
{
    let (mut data, ids): (Vec<f32>, Vec<String>) = indices
        .iter()
        .map(|i| &aes_data[*i])
        .zip(identifiers.iter())
        // filter values that are NaN
        .filter_map(|(col, id)| col.as_ref().map(|x| (*x, id.clone())))
        .unzip();
    if data.is_empty() {
        return;
    }
    for e in to_remove.iter() {
        commands.entity(e).despawn_recursive();
    }
    commands
        .spawn(aesthetics::Aesthetics {
            identifiers: ids,
            condition: if ggcomp.cond.is_empty() {
                None
            } else {
                Some(ggcomp.cond.to_string())
            },
        })
        .insert(ggcomp.aes_component)
        .insert(aesthetics::Point(std::mem::take(&mut data)))
        .insert(ggcomp.geom_component);
}

fn insert_geom_hist<F, Aes: Component, Geom: Component>(
    commands: &mut Commands,
    dist_data: &mut [Vec<Number>],
    indices: &HashSet<usize>,
    identifiers: &[String],
    to_remove: &Query<(Entity, &AesFilter), F>,
    ggcomp: GgPair<Aes, Geom>,
) where
    F: ReadOnlyWorldQuery,
{
    let (mut data, ids): (Vec<Vec<f32>>, Vec<String>) = indices
        .iter()
        .map(|i| std::mem::take(&mut dist_data[*i]))
        // also filter values that are NaN
        .zip(identifiers.iter())
        .map(|(col, id)| {
            (
                std::mem::take(
                    &mut col
                        .into_iter()
                        .filter_map(|c| c.into())
                        .collect::<Vec<f32>>(),
                ),
                id.clone(),
            )
        })
        .filter(|(c, _)| !c.is_empty())
        .unzip();
    if !data.is_empty() {
        // remove existing sizes geoms
        for (e, aes_filter) in to_remove.iter() {
            if aes_filter.met == ggcomp.met & !aes_filter.pbox {
                commands.entity(e).despawn_recursive();
            }
        }
        let mut ent_commands = commands.spawn(ggcomp.geom_component);
        ent_commands
            .insert(aesthetics::Aesthetics {
                identifiers: ids,
                condition: if ggcomp.cond.is_empty() {
                    None
                } else {
                    Some(ggcomp.cond.to_string())
                },
            })
            .insert((
                ggcomp.aes_component,
                aesthetics::Distribution(std::mem::take(&mut data)),
                AesFilter {
                    met: ggcomp.met,
                    pbox: false,
                },
            ));
        if ggcomp.hover {
            ent_commands.insert(geom::PopUp {});
        }
    }
}
