//! Input data logic.

use std::collections::HashSet;

use crate::aesthetics;
use crate::escher::EscherMap;
use crate::geom::{self, HistTag, Xaxis};
use crate::geom::{AesFilter, GeomHist, HistPlot};
use crate::info::Info;
use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use itertools::Itertools;
use serde::Deserialize;

pub struct DataPlugin;

impl Plugin for DataPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<EscherMap>()
            .init_asset::<Data>()
            .register_asset_loader(CustomAssetLoader::<EscherMap>::new(vec!["json"]))
            .register_asset_loader(CustomAssetLoader::<Data>::new(vec!["metabolism.json"]))
            .add_systems(PostUpdate, load_data);
    }
}

#[derive(Default)]
pub struct CustomAssetLoader<A> {
    extensions: Vec<&'static str>,
    _mark: std::marker::PhantomData<A>,
}

/// Possible errors that can be produced by [`CustomAssetLoader`]
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum CustomJsonLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    /// A [RON](ron) Error
    #[error("Could not parse JSON: {0}")]
    JsonSpannedError(#[from] serde_json::Error),
}

impl<A> AssetLoader for CustomAssetLoader<A>
where
    for<'de> A: serde::Deserialize<'de> + bevy::asset::Asset,
{
    type Asset = A;
    type Settings = ();
    type Error = CustomJsonLoaderError;
    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let custom_asset = serde_json::from_slice::<A>(&bytes)?;
        Ok(custom_asset)
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
    #[allow(dead_code)]
    // some libraries may use "NaN" or "Inf" as null in JSON we don't care about
    // those values but still has to be as is since serde(other) is not possible
    // for untagged enums.
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
#[derive(Deserialize, Asset, Default, TypePath)]
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

trait IsEmpty {
    fn is_empty(&self) -> bool;
}

impl<T> IsEmpty for Option<Vec<T>> {
    fn is_empty(&self) -> bool {
        self.as_ref().map(|x| x.is_empty()).unwrap_or(true)
    }
}

impl IsEmpty for Data {
    #[rustfmt::skip]
    /// [`Data`] is empty if no identifiers are passed or no numeric data is passed.
    fn is_empty(&self) -> bool {
        if self.reactions.is_empty() & self.metabolites.is_empty()
        {
            return true;
        }
        self.colors.is_empty() & self.sizes.is_empty() & self.y.is_empty() &
        self.left_y.is_empty() & self.hover_y.is_empty() & self.kde_y.is_empty() &
        self.kde_left_y.is_empty() & self.kde_hover_y.is_empty() & self.box_y.is_empty() &
        self.box_left_y.is_empty() & self.conditions.is_empty() & self.met_conditions.is_empty() &
        self.met_colors.is_empty() & self.met_sizes.is_empty() & self.met_y.is_empty() & self.kde_met_y.is_empty()
    }
}

/// Resource that contains a [`Handle`] to user data. Modified when new datas comes in.
#[derive(Resource)]
pub struct ReactionState {
    pub reaction_data: Option<Handle<Data>>,
    pub loaded: bool,
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
    mut info_state: ResMut<Info>,
    mut custom_assets: ResMut<Assets<Data>>,
    asset_server: Res<AssetServer>,
    mut restore_event: EventWriter<aesthetics::RestoreEvent>,
    // remove data to be plotted, axes and histograms
    to_remove: Query<Entity, Or<(With<aesthetics::Aesthetics>, With<HistTag>, With<Xaxis>)>>,
) {
    let custom_asset = if let Some(reac_handle) = &state.reaction_data {
        if let Some(bevy::asset::LoadState::Failed(_)) = asset_server.get_load_state(reac_handle) {
            info_state
                .notify("Failed loading data! Check if your metabolism.json is in correct format.");
            state.reaction_data = None;
            return;
        }
        custom_assets.get_mut(reac_handle.id())
    } else {
        return;
    };
    if state.loaded || custom_asset.is_none() {
        return;
    }

    let data = custom_asset.unwrap();
    if data.is_empty() {
        return;
    }
    info_state.notify("Loading data...");
    // remove all previous plotted data
    for e in to_remove.iter() {
        commands.entity(e).despawn_recursive();
    }
    restore_event.send(aesthetics::RestoreEvent {});
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

    info_state.notify("Loading Metabolite data!");
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

    state.loaded = true;
    info_state.close()
}

fn insert_geom_map<Aes: Component, Geom: Component>(
    commands: &mut Commands,
    indices: &HashSet<usize>,
    aes_data: &[Number],
    identifiers: &[String],
    ggcomp: GgPair<Aes, Geom>,
) {
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

fn insert_geom_hist<Aes: Component, Geom: Component>(
    commands: &mut Commands,
    dist_data: &mut [Vec<Number>],
    indices: &HashSet<usize>,
    identifiers: &[String],
    ggcomp: GgPair<Aes, Geom>,
) {
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
