# File Formats

For a quick start, see the [map example](https://github.com/biosustain/shu/blob/master/assets/ecoli_core_map.json) (or any escher map at
https://escher.github.io/) and the [data example](https://github.com/biosustain/shu/blob/master/assets/flux_kcat.metabolism.json).

## Map

Shu uses the same format as escher for the maps. Maps from escher can be imported
using the Map button (web app) or with drag and drop (native app).

The only difference is that the histogram position, rotation and scale (which
does not exist in escher) can be exported to the map (only native app for now)
using the `Export` drop down on the `Settings` window. This allows to save a
map with the correct manually fixed positions where different data can be
plotted for the same or different projects.

For the full JSON specification (ending with the extension _.json_), please refer
to the source code represented by the `EscherMap` struct found at [the map source code](https://github.com/biosustain/shu/blob/master/src/escher.rs).

## Data

The input data that contains the variables to be plotted in the map is a JSON ending withe _.metabolism.json_ extension. The full specification is the following struct (source code at [the data source code](https://github.com/biosustain/shu/blob/master/src/data.rs)):

```rust
pub struct Data {
    /// Vector of reactions' identifiers
    reactions: Option<Vec<String>>,
    // TODO: generalize this for any Data Type and use them (from escher.rs)
    /// Numeric values to plot as reaction arrow colors.
    colors: Option<Vec<Number>>,
    /// Numeric values to plot as reaction arrow sizes.
    sizes: Option<Vec<Number>>,
    /// Numeric values to plot as histogram.
    y: Option<Vec<Vec<Number>>>,
    /// Numeric values to plot as histogram.
    left_y: Option<Vec<Vec<Number>>>,
    /// Numeric values to plot on a hovered histogram popup.
    hover_y: Option<Vec<Vec<Number>>>,
    /// Numeric values to plot as KDE.
    kde_y: Option<Vec<Vec<Number>>>,
    /// Numeric values to plot as KDE.
    kde_left_y: Option<Vec<Vec<Number>>>,
    /// Numeric values to plot on a hovered KDE popup.
    kde_hover_y: Option<Vec<Vec<Number>>>,
    /// Numeric values to plot as boxpoint.
    box_y: Option<Vec<Number>>,
    /// Numeric values to plot as boxpoint.
    box_left_y: Option<Vec<Number>>,
    /// Vector of identifiers for horizontal ordering of the box boxpoints (right).
    box_variant: Option<Vec<String>>,
    /// Vector of identifiers for horizontal ordering of the box boxpoints (left).
    box_left_variant: Option<Vec<String>>,
    /// Numeric values to plot a column plot (right).
    column_y: Option<Vec<Number>>,
    column_ymin: Option<Vec<Number>>,
    column_ymax: Option<Vec<Number>>,
    /// Numeric values to plot a column plot (left).
    left_column_y: Option<Vec<Number>>,
    left_column_ymin: Option<Vec<Number>>,
    left_column_ymax: Option<Vec<Number>>,
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
```
