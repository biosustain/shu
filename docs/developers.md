# For developers

The following instructions are for developers who want to contribute to **shu** or deploy it.

## Build instructions from source

To build **shu** from source, you need to have Rust and `cargo` installed.
**Shu** uses the nightly toolchain: 

```bash
rustup toolchain install nightly
```

and `ldd` to speed up linking times. See the [bevy setup documentation](https://bevyengine.org/learn/book/getting-started/setup/#enable-fast-compiles-optional) to
install it or simply remove the `.cargo` directory if that is not desired.

Then, clone the repository and build the project:

```bash
git clone https://github.com/biosustain/shu.git
cd shu
cargo build
```

## Architecture

The API documentation can be found at https://docs.rs/shu.

Shu follows a Grammar of Graphics design like [ggplot](https://ggplot2.tidyverse.org/) or [plotnine](https://plotnine.readthedocs.io/en/stable/index.html). See the
[plotting chapter](plotting.md) for the full analogy. The
particular implementation is an Entity Component System in [bevy](https://bevyengine.org/):

* Each aesthetic is a *component* (`Gsize`, `Gcolor`, etc.) containing its data (see [`aesthetics.rs`](https://github.com/biosustain/shu/tree/master/src/aesthetics.rs)). Identifiers are stored in the `Aesthetic` *component*.
* *Entities* with `Aesthetic`, other aes components and Geom component (`GeomArrow`, `GeomMetabolite`, etc. in [`geom.rs`](https://github.com/biosustain/shu/tree/master/src/geom.rs)) are processed and plotted by a *system* (in [`aesthetics.rs`](https://github.com/biosustain/shu/tree/master/src/aesthetics.rs)).
* The accepted aesthetics for a given geom are made explicit in the *queries* of the *systems*.

Data handling lives in [`data.rs`](https://github.com/biosustain/shu/tree/master/src/data.rs) and [`escher.rs`](https://github.com/biosustain/shu/tree/master/src/escher.rs).

About the GUI, there are three separate pieces: the Settings window, the
histogram interactions and legend. The settings window is handled by [bevy_egui](https://github.com/mvlabat/bevy_egui)
([`gui.rs`](https://github.com/biosustain/shu/tree/master/src/gui.rs)). The histogram interactions are non-UI components spawned in
[`aesthetics.rs`](https://github.com/biosustain/shu/tree/master/src/aesthetics.rs) and handled mostly in [`gui.rs`](https://github.com/biosustain/shu/tree/master/src/gui.rs). Finally, the legend is in its
own separate [legend module](https://github.com/biosustain/shu/tree/master/src/legend) and consists on UI components in a flexbox in a
way that it is by default collapse and only the relevant legend appears once
its corresponding data is added to the map.

## Deployment

Binaries for Linux, Mac, Windows and WASM are generated once release tags are pushed to the repository. For instance, a tag for version 26.2.0 would be pushed in the following way:

```bash
git tag 26.2.0
git push origin 26.2.0
```

A github action will build and create the binaries.

### Native deployment

To generate binaries locally:

```bash
git clone https://github.com/biosustain/shu.git
cd shu
cargo build --release
```

A binary has been generated at `target/release/shu`. You can copy it to your path or run it directly from there. Alternatively, simply run

```bash
cargo install --path .
```

And [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) will handle the rest.

### WASM deployment

If you want to deploy the WASM version locally or for internal use, you can build it yourself using [wasm-bindgen](https://rustwasm.github.io/wasm-bindgen/reference/cli.html):

```bash
# clone the repository if you haven't already
git clone https://github.com/biosustain/shu.git
cd shu
# build the wasm version
cargo build --profile wasm-release --target wasm32-unknown-unknown
# bundle it into the pkg directory
wasm-bindgen --out-dir pkg --target web ./target/wasm32-unknown-unknown/wasm-release/shu.wasm
```

Afterwards, an `index.html` file can be created in the root directory that
contains `pkg`. See the [gh-pages branch](https://github.com/biostain/shu/tree/gh-pages) for an example of that. A directory
containing the `inde.html`, `assets` and `pkg` directories can be deployed to a
static page like [Github pages](https://pages.github.com/) or [Gitlab pages](https://docs.gitlab.com/ee/user/project/pages/).

## Contributing

Contributions are welcome!

1. Look up similar [issues](https://github.com/biosustain/shu/issues).
2. [Write an issue](https://github.com/biosustain/shu/issues/new).
3. [Fork](https://docs.github.com/en/enterprise/2.13/user/articles/fork-a-repo) the repository.

```bash
# https
git clone https://github.com/biosustain/shu.git
# or ssh
git clone git@github.com:biosustain/shu.git
# add a remote with to your fork
git remote add downstream git@github.com:username/shu.git
```

4. Branch from trunk.

```bash
git checkout -b 'feat-incrediblefeature'
```

5. Write your code and push the commits to you repository (we use [semantic commits](https://seesparkbox.com/foundry/semantic_commit_messages)).

```bash
git commit -m 'feat: add incredible feature'
git push -u downstream feat-incrediblefeature
```

6. Submit a [Pull Request](https://github.com/biosustain/shu/pulls) with your feature/bug fix.
7. Get the Pull Request approved (CI must pass).  

For the CI to pass, you need to run and pass `cargo fmt` and `cargo test`
before comitting. `cargo clippy` is not enforced but desirable.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as Apache2.0/MIT, without any additional terms or conditions.
