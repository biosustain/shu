For developers
==============

The following instructions are for developers who want to contribute to **shu** or deploy it.

Build instructions from source
------------------------------

To build **shu** from source, you need to have Rust and `cargo` installed.
**Shu** uses the nightly toolchain: 

.. code:: bash

  rustup toolchain install nightly

and `ldd` to speed up linking times. See the `bevy setup documentation`_ to
install it or simply remove the `.cargo` directory if that is not desired.

Then, clone the repository and build the project:

.. code:: bash

  git clone https://github.com/biosustain/shu.git
  cd shu
  cargo build

Architecture
------------

The API documentation can be found at https://docs.rs/shu.

Shu follows a Grammar of Graphics design like ggplot_  or plotnine_. See the
`plotting chapter <plotting.html>`__ for the full analogy. The
particular implementation is an Entity Component System in bevy_:

* Each aesthetic is a *component* (`Gsize`, `Gcolor`, etc.) containing its data (see `aesthetics.rs`_). Identifiers are stored in the `Aesthetic` *component*.
* *Entities* with `Aesthetic`, other aes components and Geom component (`GeomArrow`, `GeomMetabolite`, etc. in `geom.rs`_) are processed and plotted by a *system* (in `aesthetics.rs`_).
* The accepted aesthetics for a given geom are made explicit in the *queries* of the *systems*.

Data handling lives in `data.rs`_ and `escher.rs`_.

About the GUI, there are three separate pieces: the Settings window, the
histogram interactions and legend. The settings window is handled by bevy_egui_
(`gui.rs`_). The histogram interactions are non-UI components spawned in
`aesthetics.rs`_ and handled mostly in `gui.rs`_. Finally, the legend is in its
own separate `legend module`_ and consists on UI components in a flexbox in a
way that it is by default collapse and only the relevant legend appears once
its corresponding data is added to the map.

Deployment
----------

Binaries for Linux, Mac, Windows and WASM are generated once release tags are pushed to the repository. For instance, a tag for version 26.2.0 would be pushed in the following way:

.. code:: bash

  git tag 26.2.0
  git push origin 26.2.0

A github action will build and create the binaries.

Native deployment
~~~~~~~~~~~~~~~~~

To generate binaries locally:

.. code:: bash

  git clone https://github.com/biosustain/shu.git
  cd shu
  cargo build --release

A binary has been generated at `target/release/shu`. You can copy it to your path or run it directly from there. Alternatively, simply run

.. code:: bash

   cargo install --path .

And `cargo`_ will handle the rest.

WASM deployment
~~~~~~~~~~~~~~~

If you want to deploy the WASM version locally or for internal use, you can build it yourself using `wasm-bindgen`_:

.. code:: bash

  # clone the repository if you haven't already
  git clone https://github.com/biosustain/shu.git
  cd shu
  # build the wasm version
  cargo build --profile wasm-release --target wasm32-unknown-unknown
  # bundle it into the pkg directory
  wasm-bindgen --out-dir pkg --target web ./target/wasm32-unknown-unknown/wasm-release/shu.wasm

Afterwards, an `index.html` file can be created in the root directory that
contains `pkg`. See the `gh-pages branch`_ for an example of that. A directory
containing the `inde.html`, `assets` and `pkg` directories can be deployed to a
static page like `Github pages`_ or `Gitlab pages`_.

Contributing
------------

Contributions are welcome!

1. Look up similar issues_.
2. `Write an issue`_.
3. Fork_ the repository.

.. code:: bash

  # https
  git clone https://github.com/biosustain/shu.git
  # or ssh
  git clone git@github.com:biosustain/shu.git
  # add a remote with to your fork
  git remote add downstream git@github.com:username/shu.git

4. Branch from trunk.

.. code:: bash

  git checkout -b 'feat-incrediblefeature'

5. Write your code and push the commits to you repository (we use `semantic commits`_).

.. code:: bash

  git commit -m 'feat: add incredible feature'
  git push -u downstream feat-incrediblefeature

6. Submit a `Pull Request`_ with your feature/bug fix.
7. Get the Pull Request approved (CI must pass).  

For the CI to pass, you need to run and pass `cargo fmt` and `cargo test`
before comitting. `cargo clippy` is not enforced but desirable.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as Apache2.0/MIT, without any additional terms or conditions.

.. _cargo: https://doc.rust-lang.org/cargo/getting-started/installation.html
.. _ggplot: https://ggplot2.tidyverse.org/
.. _plotnine: https://plotnine.readthedocs.io/en/stable/index.html
.. _python API: https://vita.had.co.nz/papers/tidy-data.html 
.. _bevy: https://bevyengine.org/
.. _bevy setup documentation: https://bevyengine.org/learn/book/getting-started/setup/#enable-fast-compiles-optional
.. _bevy_egui: https://github.com/mvlabat/bevy_egui
.. _aesthetics.rs: https://github.com/biosustain/shu/tree/master/src/aesthetics.rs
.. _geom.rs: https://github.com/biosustain/shu/tree/master/src/geom.rs
.. _data.rs: https://github.com/biosustain/shu/tree/master/src/data.rs
.. _escher.rs: https://github.com/biosustain/shu/tree/master/src/escher.rs
.. _gui.rs: https://github.com/biosustain/shu/tree/master/src/gui.rs
.. _legend module: https://github.com/biosustain/shu/tree/master/src/legend
.. _wasm-bindgen: https://rustwasm.github.io/wasm-bindgen/reference/cli.html
.. _gh-pages branch: https://github.com/biostain/shu/tree/gh-pages
.. _Github pages: https://pages.github.com/
.. _Gitlab pages: https://docs.gitlab.com/ee/user/project/pages/
.. _issues: https://github.com/biosustain/shu/issues
.. _Write an issue: https://github.com/biosustain/shu/issues/new
.. _Fork: https://docs.github.com/en/enterprise/2.13/user/articles/fork-a-repo
.. _semantic commits: https://seesparkbox.com/foundry/semantic_commit_messages 
.. _Pull request: https://github.com/biosustain/shu/pulls
