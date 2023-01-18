# shu

<img align="right" width="172" height="228" src="./assets/logo.svg">

## What?

App to plot multidimensional data to a metabolic map. Metabolic maps are graphs with metabolites
as nodes and reactions as edges.

## Why?

[Escher](https://escher.github.io/#/) is great. In fact, the default look of the map and the format
of the map are exactly the same as escher's. However, escher only allows for plotting 2 (+2 with tooltips)
kinds of data: reaction data and metabolite data. **Shu** attempts to provide ways of plotting at least
6:

- [x] Reaction sizes.
- [x] Reaction colors.
- [x] Reaction right sides.
- [x] Reaction left sides.
- [x] Metabolite sizes.
- [x] Metabolite colors.

(+2 with hovers):
- [x] Hover reactions.
- [x] Hover metabolites.

with special focus on being able to plot **distributions** (not just points) and **n-conditions**. Escher also has the
distinction between color and size, it is simply that they are not independently accessible from the GUI.


## API design
 
Shu follows a Grammar of Graphics design like [ggplot](https://ggplot2.tidyverse.org/) or [plotnine](https://plotnine.readthedocs.io/en/stable/index.html).
See the [python API](ggshu/README.rst) for the full analogy. The particular implementation
is an Entity Component System in [bevy](https://bevyengine.org/):

* Each aesthetic is a *component* (`Gsize`, `Gcolor`, etc.) containing its data (see [`src/aesthetics.rs`](src/aesthetics.rs)). Identifiers are stored in the `Aesthetic` *component*.
* *Entities* with `Aesthetic`, other aes components and Geom component (`GeomArrow`, `GeomMetabolite`, etc. in [`src/geom.rs`](src/geom.rs)) are
processed and plotted by a *system* (in [`src/aesthetics.rs`](src/aesthetics.rs)).
* The accepted aesthetics for a given geom are made explicit in the *queries* of the *systems*.

Data handling (`df`, ad `map_file`) lives in [`src/data.rs`](src/data.rs) and
[`src/escher.rs`](src/escher.rs) and the GUI componets lives in [`src/gui.rs`](src/gui.rs).

## License

Copyright 2023 The Novo Nordisk Foundation Center for Biosustainability.

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
