Welcome to shu's documentation!
===============================

**shu** is an interface to plot data on top of metabolic networks,
with emphasis in multi-dimensional data across diferent conditions and distributions.

Shu is available as a web application at https://biosustain.github.io/shu or as a native
application (see `releases`_).

Check out the :doc:`plotting` section for an overview on how to generate and plot data
into the map. The :doc:`file_formats` explains the map (with is fully compatible
with `escher`_) and data especification.

Controls
--------

* **Left click** and drag to move around the map.
* **Scroll whell** to zoom in and out.
* **Middle click** on a histogram or the legend (on its center) and drag the mouse while holding
  the button to move it.
* **Right click** on a histogram or the legend (on its center) and drag the mouse while holding
  the button to zoom in/out to rotate it.
* **Right click + Shift** on a histogram (on its center) and drag the mouse while holding
  the button to scale the x-axis of the histogram.
* :code:`+` and :code:`-` keys to scale up and down the legend. If :code:`Control` is pressed,
  the Settings are scale.
* The `Settings` window can be used to change the appeareance and export the map as a json or an image.

Contents
--------

.. toctree::

   plotting
   file_formats
   developers

.. _releases: https://github.com/biosustain/shu/releases/latest
.. _escher: https://github.com/zakandrewking/escher
