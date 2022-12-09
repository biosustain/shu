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

#[derive(Resource)]
pub struct ReactionState {
    pub reaction_data: Option<Handle<Data>>,
    pub reac_loaded: bool,
    pub met_loaded: bool,
}

#[allow(clippy::too_many_arguments)]
fn load_data(
    mut commands: Commands,
    mut state: ResMut<ReactionState>,
    mut custom_assets: ResMut<Assets<Data>>,
    current_sizes: Query<Entity, (With<aesthetics::Gsize>, With<geom::GeomArrow>)>,
    current_colors: Query<Entity, (With<aesthetics::Gcolor>, With<geom::GeomArrow>)>,
    current_hist: Query<Entity, Or<(With<geom::GeomHist>, With<geom::HistTag>)>>,
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
    let conditions = data.conditions.clone().unwrap_or(vec![String::from("")]);
    let cond_set = conditions.iter().unique().collect::<HashSet<&String>>();
    if let Some(reactions) = data.reactions.as_ref() {
        for cond in cond_set.iter() {
            let indices: HashSet<usize> = if cond.is_empty() & (conditions.len() == 1) {
                data.reactions
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
            for (i, var) in [&mut data.colors, &mut data.sizes].iter().enumerate() {
                if let Some(point_data) = &var {
                    let (mut data, ids): (Vec<f32>, Vec<String>) = indices
                        .iter()
                        .map(|i| &point_data[*i])
                        .zip(identifiers.iter())
                        // filter values that are NaN
                        .filter_map(|(col, id)| col.as_ref().map(|x| (*x, id.clone())))
                        .unzip();
                    if !data.is_empty() {
                        // remove existing color geoms
                        if i == 0 {
                            for e in current_colors.iter() {
                                commands.entity(e).despawn_recursive();
                            }
                        } else {
                            for e in current_sizes.iter() {
                                commands.entity(e).despawn_recursive();
                            }
                        }
                        let mut build_command = commands.spawn(aesthetics::Aesthetics {
                            plotted: false,
                            identifiers: ids,
                            condition: if cond.is_empty() {
                                None
                            } else {
                                Some(cond.to_string())
                            },
                        });
                        build_command
                            .insert(aesthetics::Point(std::mem::take(&mut data)))
                            .insert(geom::GeomArrow { plotted: false });
                        if i == 0 {
                            build_command.insert(aesthetics::Gcolor {});
                        } else {
                            build_command.insert(aesthetics::Gsize {});
                        }
                    }
                }
            }
            for (i, aes) in [
                &mut data.y,
                &mut data.left_y,
                &mut data.kde_y,
                &mut data.kde_left_y,
                &mut data.hover_y,
                &mut data.kde_hover_y,
            ]
            .iter_mut()
            .enumerate()
            {
                if let Some(dist_data) = aes.as_mut() {
                    let (mut data, ids): (Vec<Vec<f32>>, Vec<String>) = indices
                        .iter()
                        .map(|i| std::mem::take(&mut dist_data[*i]))
                        // also filter values that are NaN
                        .zip(identifiers.iter())
                        .map(|(col, id)| {
                            (
                                std::mem::take(
                                    &mut col.into_iter().filter_map(|c| c.into()).collect(),
                                ),
                                id.clone(),
                            )
                        })
                        .unzip();
                    data.retain(|c| !c.is_empty());
                    if data.is_empty() {
                        continue;
                    }
                    // remove existing sizes geoms
                    for e in current_hist.iter() {
                        commands.entity(e).despawn_recursive();
                    }
                    let geom = match i {
                        0 => geom::GeomHist::right(geom::HistPlot::Hist),
                        1 => geom::GeomHist::left(geom::HistPlot::Hist),
                        2 => geom::GeomHist::right(geom::HistPlot::Kde),
                        3 => geom::GeomHist::left(geom::HistPlot::Kde),
                        4 => geom::GeomHist::up(geom::HistPlot::Hist),
                        _ => geom::GeomHist::up(geom::HistPlot::Kde),
                    };
                    let mut ent_commands = commands.spawn(aesthetics::Gy {});
                    ent_commands
                        .insert(aesthetics::Distribution(std::mem::take(&mut data)))
                        .insert(geom)
                        .insert(aesthetics::Aesthetics {
                            plotted: false,
                            identifiers: ids,
                            condition: if cond.is_empty() {
                                None
                            } else {
                                Some(cond.to_string())
                            },
                        });
                    // for hovers
                    if i > 3 {
                        ent_commands.insert(geom::PopUp {});
                    }
                }
            }
            for (i, var) in [&mut data.box_y, &mut data.box_left_y].iter().enumerate() {
                if let Some(point_data) = var {
                    let (mut data, ids): (Vec<f32>, Vec<String>) = indices
                        .iter()
                        .map(|i| &point_data[*i])
                        .zip(identifiers.iter())
                        // filter values that are NaN
                        .filter_map(|(col, id)| col.as_ref().map(|x| (*x, id.clone())))
                        .unzip();
                    // remove existing sizes geoms
                    for e in current_hist.iter() {
                        commands.entity(e).despawn_recursive();
                    }
                    let geom = match i {
                        0 => geom::GeomHist::right(geom::HistPlot::BoxPoint),
                        _ => geom::GeomHist::left(geom::HistPlot::BoxPoint),
                    };
                    commands
                        .spawn(aesthetics::Gy {})
                        .insert(aesthetics::Point(std::mem::take(&mut data)))
                        .insert(geom)
                        .insert(aesthetics::Aesthetics {
                            plotted: false,
                            identifiers: ids,
                            condition: if cond.is_empty() {
                                None
                            } else {
                                Some(cond.to_string())
                            },
                        });
                }
            }
        }
    }

    info!("Loading Metabolite data!");
    if let Some(metabolites) = data.metabolites.as_ref() {
        for cond in cond_set {
            let indices: HashSet<usize> = if cond.is_empty() & (conditions.len() == 1) {
                data.reactions
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
                let (mut data, ids): (Vec<f32>, Vec<String>) = indices
                    .iter()
                    .map(|i| &color_data[*i])
                    .zip(identifiers.iter())
                    // filter values that are NaN
                    .filter_map(|(col, id)| col.as_ref().map(|x| (*x, id.clone())))
                    .unzip();
                for e in current_met_colors.iter() {
                    commands.entity(e).despawn_recursive();
                }
                commands
                    .spawn(aesthetics::Aesthetics {
                        plotted: false,
                        identifiers: ids,
                        condition: if cond.is_empty() {
                            None
                        } else {
                            Some(cond.to_string())
                        },
                    })
                    .insert(aesthetics::Gcolor {})
                    .insert(aesthetics::Point(std::mem::take(&mut data)))
                    .insert(geom::GeomMetabolite { plotted: false });
            }
            if let Some(size_data) = &mut data.met_sizes {
                let (mut data, ids): (Vec<f32>, Vec<String>) = indices
                    .iter()
                    .map(|i| &size_data[*i])
                    .zip(identifiers.iter())
                    // filter values that are NaN
                    .filter_map(|(col, id)| col.as_ref().map(|x| (*x, id.clone())))
                    .unzip();

                // remove existing sizes geoms
                for e in current_met_sizes.iter() {
                    commands.entity(e).despawn_recursive();
                }
                commands
                    .spawn(aesthetics::Aesthetics {
                        plotted: false,
                        identifiers: ids,
                        condition: if cond.is_empty() {
                            None
                        } else {
                            Some(cond.to_string())
                        },
                    })
                    .insert(aesthetics::Gsize {})
                    .insert(aesthetics::Point(std::mem::take(&mut data)))
                    .insert(geom::GeomMetabolite { plotted: false });
            }
            if let Some(dist_data) = &mut data.met_y {
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
                    for e in current_sizes.iter() {
                        commands.entity(e).despawn_recursive();
                    }
                    commands
                        .spawn(aesthetics::Aesthetics {
                            plotted: false,
                            identifiers: ids,
                            condition: if cond.is_empty() {
                                None
                            } else {
                                Some(cond.to_string())
                            },
                        })
                        .insert(aesthetics::Gy {})
                        .insert(aesthetics::Distribution(std::mem::take(&mut data)))
                        .insert(geom::PopUp {})
                        .insert(geom::GeomHist::up(geom::HistPlot::Hist));
                }
            }
            if let Some(dist_data) = &mut data.kde_met_y {
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
                    for e in current_sizes.iter() {
                        commands.entity(e).despawn_recursive();
                    }
                    commands
                        .spawn(aesthetics::Aesthetics {
                            plotted: false,
                            identifiers: ids,
                            condition: if cond.is_empty() {
                                None
                            } else {
                                Some(cond.to_string())
                            },
                        })
                        .insert(aesthetics::Gy {})
                        .insert(aesthetics::Distribution(std::mem::take(&mut data)))
                        .insert(geom::PopUp {})
                        .insert(geom::GeomHist::up(geom::HistPlot::Kde));
                }
            }
        }
    }

    state.met_loaded = true;
    state.reac_loaded = true;
}
