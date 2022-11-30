ggshu
=====

  This is alpha software and very subject to change!

A utility package to transform dataframes into `shu <https://github.com/biosustain/shu/>`_ data.


Example
-------

.. code-block:: python

    (
        ggmap(
            df_cond,
            aes(reaction="r", color="flux", size="flux", condition="cond"),
        )
        # plot flux to color and size of reactions
        + geom_arrow()
        # plot conc to color of metabolites
        + geom_metabolite(aes=aes(color="conc", metabolite="m"))
        # plot km as density plots shows on hover on metabolites
        + geom_kde(aes=aes(y="km"), mets=True)
    ).to_json("shu_data")
