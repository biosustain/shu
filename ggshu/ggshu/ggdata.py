"""Main class that performs data wrangle and build the final plotting data for shu."""

from __future__ import annotations
import logging
import json
from math import isnan

import pandas as pd
from ggshu.aes import Aesthetics
from ggshu.geoms import Geom

LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(logging.DEBUG)


class PlotData:
    """Main class of ggshu, aliases as ggmap().

    Parameters
    ----------
    df: pd.DataFrame
        Data frame in tidy format.
    aes: Aesthetics
        created by `aes()`.

    Example
    -------

    ```python
    import pandas as pd
    from ggshu import aes, geom_arrow, ggmap

    df = pd.DataFrame({"reaction": ["PFK", "ENO"], "flux": [2, 4]})
    (ggmap(df, aes(reaction="reaction", color="flux")) + geom_arrow()).to_json("shu_data")
    ```

    """

    def __init__(self, df: pd.DataFrame, aes: Aesthetics):
        self.aes = aes
        reac_grouping = [
            aes[variable] for variable in ["reaction", "condition"] if variable in aes
        ]
        met_grouping = [
            aes[variable] for variable in ["metabolite", "condition"] if variable in aes
        ]
        self.passed_df = df
        self.df_reac = None
        self.df_met = None
        self.plotting_data = {}
        if "reaction" in aes:
            self.df_reac: pd.DataFrame = (
                df.groupby(reac_grouping).agg(list).reset_index()
            )
            self.plotting_data["reactions"] = self.df_reac[aes["reaction"]]
            if "condition" in aes:
                self.plotting_data["conditions"] = self.df_reac[aes["condition"]]
        if "metabolite" in aes:
            self.df_met: pd.DataFrame = df.groupby(met_grouping).agg(list).reset_index()
            self.plotting_data["metabolites"] = self.df_met[aes["metabolite"]]
            if "condition" in aes:
                self.plotting_data["met_conditions"] = self.df_met[aes["condition"]]

    def __add__(self, other: Geom) -> PlotData:
        """Add a geom to be plotted."""
        if any("met" in val for val in other.mapping.values()):
            if other.aes is not None:
                if "metabolite" in other.aes and "metabolite" in self.aes:
                    LOGGER.warning(
                        "Overwriting metabolite aesthetics.\n"
                        "Metabolite aesthetics has to be unique in the map!"
                    )
                if "metabolite" in other.aes and other.df is None:
                    met_grouping = [other.aes["metabolite"]]
                    if "condition" in other.aes:
                        met_grouping.append(other.aes["condition"])
                        self.plotting_data["met_conditions"] = self.passed_df[other.aes["condition"]]
                    elif "condition" in self.aes:
                        met_grouping.append(self.aes["condition"])
                        self.plotting_data["met_conditions"] = self.passed_df[self.aes["condition"]]
                    self.df_met: pd.DataFrame = (
                        self.passed_df.groupby(met_grouping).agg(list).reset_index()
                    )
                    self.plotting_data["metabolites"] = self.df_met[
                        other.aes["metabolite"]
                    ]
                elif "metabolite" in other.aes:
                    self.plotting_data["metabolites"] = other.df[
                        other.aes["metabolite"]
                    ]
            self.plotting_data.update(other.map(self.df_met, self.aes))
        else:
            if other.aes is not None:
                if "reaction" in other.aes and "reaction" in self.aes:
                    LOGGER.warning(
                        "Overwriting reaction aesthetics.\n"
                        "Reaction aesthetics has to be unique in the map!"
                    )
                if "reaction" in other.aes and other.df is None:
                    self.plotting_data["reactions"] = self.df_reac[self.aes["reaction"]]
                elif "reaction" in other.aes:
                    self.plotting_data["reactions"] = other.df[other.aes["reaction"]]
            self.plotting_data.update(other.map(self.df_reac, self.aes))
        return self


    def __truediv__(self, other: PlotData) -> PlotData:
        """Combine two `PlotData`.

        ```python

        (
            (ggplot(df_reac, aes(reaction="reaction", y="flux")) + geom_hist())
            / (ggplot(df_met, aes(metabolite="metabolite", color="concentration")) + geom_metabolite())
        ).to_json("shu_data")
        ```

        """
        self.plotting_data.update(other.plotting_data)
        return self


    def to_json(self, json_file_without_extension: str):
        """Write to shu data to JSON.

        This file can be the dragged and dropped into shu to visualize
        data.

        Parameters
        ----------
        json_file_without_extension: str
            Path to desired destination. It should not contain the extension
            since the final file has to contain a "metabolism.json" extension
            for shu to parse it. This way, we enforce that particularity.

        Example
        -------

        ```python
        import pandas as pd
        from ggshu import aes, geom_arrow, ggmap

        df = pd.DataFrame({"met": ["glc", "akg"], "conc": [4, 10]})
        (ggmap(df, aes(metabolite="met", size="conc")) + geom_map()).to_json("shu_data")
        ```
        """
        json_file = json_file_without_extension + ".metabolism.json"

        shu_data = {k: v.to_list() for k, v in self.plotting_data.items()}
        for key, values in shu_data.items():
            if key not in ["reactions", "conditions", "metabolites", "met_conditions"]:
                for i in range(len(values)):
                    if isinstance(values[i], list):
                        shu_data[key][i] = [
                            v if not isnan(v) else "NaN" for v in values[i]
                        ]
                    else:
                        shu_data[key][i] = values[i] if not isnan(values[i]) else "NaN"
            with open(json_file, "w") as f:
                json.dump(shu_data, f)
