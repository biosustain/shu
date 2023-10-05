File Formats
============

For a quick start, see the `map example`_ (or any escher map at
https://escher.github.io/) and the `data example`_.

Map
---

Shu uses the same format as escher for the maps. Maps from escher can be imported
using the Map button (web app) or with drag and drop (native app).

The only difference is that the histogram position, rotation and scale (which
does not exist in escher) can be exported to the map (only native app for now)
using the `Export` drop down on the `Settings` window. This allows to save a
map with the correct manually fixed positions where different data can be
plotted for the same or different projects.

For the full JSON specification (ending with the extension ".json"), please refer
to the source code represented by the `EscherMap` struct found at `the map source code`_.

Data
----

The input data that contains the variables to be plotted in the map is a JSON ending withe ".metabolism.json" extension. The full specification is the following struct (source code at `the data source code`_):

.. code-block:: rust

  struct Data {
      /// Vector of reactions' identifiers
      reactions: Option<Vec<String>>,
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
      /// Numeric values to plot as metabolite circle colors.
      met_colors: Option<Vec<Number>>,
      /// Numeric values to plot as metabolite circle sizes.
      met_sizes: Option<Vec<Number>>,
      /// Numeric values to plot as histogram on hover.
      met_y: Option<Vec<Vec<Number>>>,
      /// Numeric values to plot as density on hover.
      kde_met_y: Option<Vec<Vec<Number>>>,
  }


.. _map example: https://github.com/biosustain/shu/blob/master/assets/ecoli_core_map.json 
.. _data example: https://github.com/biosustain/shu/blob/master/assets/flux_kcat.metabolism.json 
.. _the map source code: https://github.com/biosustain/shu/blob/master/src/escher.rs
.. _the data source code: https://github.com/biosustain/shu/blob/master/src/data.rs
