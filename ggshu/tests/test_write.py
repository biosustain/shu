import json
import os
from ggshu import aes, geom_arrow, geom_kde, ggmap, geom_metabolite


def test_writing_does_not_raise(df_cond):
    file_name = "some_tmp_data"
    (
        ggmap(
            df_cond,
            aes(reaction="r", color="flux", size="flux", y="kcat", condition="cond"),
        )
        + geom_arrow()
        + geom_metabolite(aes=aes(color="conc", metabolite="m"))
        + geom_kde(aes=aes(y="km"), mets=True)
    ).to_json(file_name)
    assert os.path.exists(file_name + ".metabolism.json")
    with open(file_name + ".metabolism.json") as f:
        data = json.load(f)
        data = list(data.keys())
        assert all(
            a in data
            for a in [
                "colors",
                "sizes",
                "conditions",
                "reactions",
                "metabolites",
                "kde_met_y",
            ]
        ), f"Should contain all aes: {data}"
    os.remove(file_name + ".metabolism.json")


def test_writing_tmp(df_cond):
    file_name = "some_tmp_data"
    df_reac = df_cond[["r", "flux", "kcat", "cond"]]
    df_reac = df_reac[~df_reac.r.isna()]
    df_met = df_cond[["m", "conc", "km", "cond"]]
    df_met = df_met[~df_met.m.isna()]
    (
        (
            ggmap(
                df_reac,
                aes(
                    reaction="r", color="flux", size="flux", y="kcat", condition="cond"
                ),
            )
            + geom_arrow()
        )
        / (
            ggmap(
                df_met,
                aes(color="conc", metabolite="m", condition="cond"),
            )
            + geom_metabolite()
            + geom_kde(aes=aes(y="km"), mets=True)
        )
    ).to_json(file_name)
    assert os.path.exists(file_name + ".metabolism.json")
    os.remove(file_name + ".metabolism.json")
