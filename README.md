# shu

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
- [ ] Hover reactions.
- [ ] Hover metabolites.

with special focus on being able to plot **distributions** (not just points) and **n-conditions**. Escher also has the
distinction between color and size, it is simply that they are not independently accessible from the GUI.


## API design

This is how the python API (not implemented) should look:

```r
# reaction and metabolite aes correspond to their identifiers
ggshu(df, map_file, aes(metabolite=metabolite, reaction=reaction, size=flux, x=kcat)) +
  geom_arrow() +  # will use size and reaction
  geom_metabolite(aes(size=concentration)) +  # will use size and metabolite
  geom_box() +  # will use reaction and kcat, boxplot at one right of reactions
  geom_kde(aes(x=km), side="left") +  # will use reaction and kms, plotted on the other side
  scale_color_continuous(min="blue", max="red") +
  scale_circle_size(min=10., max=45.)
```

It also gives and impression of how the code is written:

* Each aesthetic is a *component* (`Gsize`, `Gcolor`, etc.) containing its data (see [`src/aesthetics.rs`](src/aesthetics.rs)). Identifiers are stored in the `Aesthetic` *component*.
* *Entities* with `Aesthetic`, other aes components and Geom component (`GeomArrow`, `GeomMetabolite`, etc. in [`src/geom.rs`](src/geom.rs)) are
processed and plotted by a *system* (in [`src/aesthetics.rs`](src/aesthetics.rs)).
* The accepted aesthetics for a given geom are made explicit in the *queries* of the *systems*.

Data handling (`df`, ad `map_file`) lives in [`src/data.rs`](src/data.rs) and
[`src/escher.rs`](src/escher.rs) and the GUI componets lives in [`src/gui.rs`](src/gui.rs).
