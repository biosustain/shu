"""Tests for highest level API."""

import pytest
import numpy as np
from ggshu import aes, geom_arrow, geom_kde, ggmap, geom_metabolite, geom_boxpoint


def test_ggmap_can_be_built(df):
    _ = (
        ggmap(
            df, aes(reaction="r", color="flux", size="flux", y="kcat", metabolite="m")
        )
        + geom_arrow()
        + geom_kde(side="left")
    )


def test_plotting_dist_data_is_coerced(df):
    plotting_data = (
        ggmap(
            df, aes(reaction="r", color="flux", size="flux", y="kcat", metabolite="m")
        )
        + geom_arrow()
        + geom_metabolite(aes=aes(color="conc"))
        + geom_kde(side="left")
    ).plotting_data
    assert plotting_data["colors"].name, "flux"
    assert isinstance(plotting_data["colors"].to_list()[0], float)
    assert plotting_data["colors"].name, "conc"
    assert isinstance(plotting_data["met_colors"].to_list()[0], float)


def test_plotting_data_has_expected_keys(df):
    plotting_data = (
        ggmap(
            df, aes(reaction="r", color="flux", size="flux", y="kcat", metabolite="m")
        )
        + geom_arrow()
        + geom_metabolite(aes=aes(color="conc"))
        + geom_kde(side="left")
    ).plotting_data
    assert [
        key in plotting_data for key in ["kde_left_y", "colors", "sizes", "met_colors"]
    ]
    assert (
        "met_sizes" not in plotting_data
    ), "Sizes should not have passed to metabolites"


def test_plotting_metabolites_are_correctly_added_from_geoms(df):
    plotting_data = (
        ggmap(df, aes(reaction="r", color="flux", size="flux", y="kcat"))
        + geom_arrow()
        + geom_metabolite(aes=aes(color="conc", metabolite="m"))
        + geom_boxpoint(side="left", aes=aes(stack="iso"))
    ).plotting_data
    assert [
        key in plotting_data
        for key in [
            "kde_left_y",
            "colors",
            "sizes",
            "met_colors",
            "metabolites",
            "reactions",
        ]
    ]
    assert (
        "met_sizes" not in plotting_data
    ), "Sizes should not have passed to metabolites"


def test_passing_conditions_to_geoms_raises(df_cond):
    with pytest.raises(AssertionError):
        _ = (
            ggmap(df_cond, aes(reaction="r", color="flux", size="flux", y="kcat"))
            + geom_arrow()
            + geom_metabolite(aes=aes(color="conc", metabolite="m", condition="flux"))
        )


def test_mixed_conditions_one_dataframe_works(df_cond):
    plotting_data = (
        ggmap(
            df_cond,
            aes(reaction="r", color="flux", size="flux", y="kcat", condition="cond"),
        )
        + geom_arrow()
        + geom_metabolite(aes=aes(color="conc", metabolite="m"))
        + geom_kde(aes=aes(y="km"), mets=True)
    ).plotting_data
    assert len(plotting_data["conditions"]) == 6
    assert len(plotting_data["met_conditions"]) == 14
    assert len(plotting_data["kde_met_y"]) == 12
    assert (
        plotting_data["kde_met_y"].apply(lambda x: len(x) == 1 and np.isnan(x[0]))
    ).sum() == 10
    assert plotting_data["colors"].name, "flux"


def test_mixed_conditions_two_dataframe_works(df_cond):
    df_reac = df_cond[["r", "flux", "kcat", "cond"]]
    df_reac = df_reac[~df_reac.r.isna()]
    df_met = df_cond[["m", "conc", "km", "cond"]]
    df_met = df_met[~df_met.m.isna()]
    plotting_data = ((
        ggmap(
            df_reac,
            aes(reaction="r", color="flux", size="flux", y="kcat", condition="cond"),
        )
        + geom_arrow()
    ) / (
        ggmap(
            df_met,
            aes(color="conc", metabolite="m", condition="cond"),
        )
        + geom_metabolite()
        + geom_kde(aes=aes(y="km"), mets=True)
    )).plotting_data
    assert len(plotting_data["conditions"]) == 6
    assert len(plotting_data["met_conditions"]) == 12
    assert len(plotting_data["kde_met_y"]) == 12
    assert (
        plotting_data["kde_met_y"].apply(lambda x: len(x) == 1 and np.isnan(x[0]))
    ).sum() == 10
    assert plotting_data["colors"].name, "flux"
    assert isinstance(plotting_data["colors"].to_list()[0], float)
    assert isinstance(plotting_data["colors"].to_list()[0], float)
