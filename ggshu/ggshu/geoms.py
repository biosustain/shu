"""Classes to map aesthetics to geometric objects."""

import logging
from typing import Dict, Optional

import numpy as np
import pandas as pd

from ggshu.aes import Aesthetics

LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(logging.DEBUG)


class Geom:
    """Abstract class for all `geom_*` functions."""

    def __init__(
        self, df: Optional[pd.DataFrame] = None, aes: Optional[Aesthetics] = None
    ):
        self.df = df
        # mapping from df-variables to aes-variables
        self.aes = aes
        # mapping from aes-variables to shu-variables
        self.mapping: Dict[str, str] = {"y": "y"}
        if self.df is not None:
            assert NotImplemented("Use only one dataframe, please!")

    def post_init(self):
        """Check that the mappings and aesthetics agree."""
        if self.aes is not None:
            assert all(
                aes in self.mapping
                for aes in self.aes
                if aes not in ["metabolite", "reaction"]
            ), (
                "Some of the aes passed directly passed to the geom are "
                f"incompatible with it! Supported: {list(self.mapping.keys())}"
            )

    def map(self, df: pd.DataFrame, aes: Aesthetics):
        """Convert the information in the df to the structure in shu."""
        df_in_use = df if self.df is None else self.df
        aes_in_use = aes if self.aes is None else self.aes
        # at least one of the specified aesthetics should be there
        assert any(
            var in aes_in_use for var in self.mapping
        ), f"This geom requires aes {self.mapping.keys()} to be specifies"
        return {
            shu_var: self.check_type(df_in_use[aes_in_use[aes_var]])
            for aes_var, shu_var in self.mapping.items()
            if aes_var in aes_in_use
        }

    def check_type(self, data: pd.Series) -> pd.Series:
        """Check and validate the datatype."""
        # the default checks for a list of lists of numbers (distribution)
        assert data.dtype == "O", "Data should be arrays"
        assert isinstance(data[0], list), "Each row should contain a list"
        assert isinstance(data[0][0], int) or isinstance(
            data[0][0], float
        ), "Each list should contain numbers"
        return data


class GeomHist(Geom):
    """Geometric mapping from aesthetics to histograms in the metabolic map.

    Parameters
    ----------
    aes: Optional[Aesthetics]
        with accepted aesthetics being `{"reaction", "metabolite", "y"}`.
    side: str, default="right"
        Either "left", "right" or "hover". It determines the placement of the geom
        with respect to the reaction
    mets: bool, default=False
        Whether the geom maps to metabolites (True) or reactions (False).
    """

    def __init__(
        self,
        *,
        df: Optional[pd.DataFrame] = None,
        aes: Optional[Aesthetics] = None,
        side="right",
        mets=False,
    ):
        super().__init__(df, aes)
        self.mapping = {
            "y": "met_y"
            if mets
            else "left_y"
            if side == "left"
            else "hover_y"
            if side == "hover"
            else "y"
        }
        self.data_property = list
        self.post_init()


class GeomKde(Geom):
    """Geometric mapping from aesthetics to a density in the metabolic map.

    It uses a standard normal kernel density function.

    Parameters
    ----------
    aes: Optional[Aesthetics]
        with accepted aesthetics being `{"reaction", "metabolite", "y"}`.
    side: str, default="right"
        Either "left", "right" or "hover". It determines the placement of the geom
        with respect to the reaction
    mets: bool, default=False
        Whether the geom maps to metabolites (True) or reactions (False).
    """

    def __init__(
        self,
        *,
        df: Optional[pd.DataFrame] = None,
        aes: Optional[Aesthetics] = None,
        side="right",
        mets=False,
    ):
        super().__init__(df, aes)
        self.mapping = {
            "y": "kde_met_y"
            if mets
            else "kde_left_y"
            if side == "left"
            else "kde_hover_y"
            if side == "hover"
            else "kde_y"
        }
        self.post_init()


class GeomArrow(Geom):
    """Geometric mapping from aesthetics to the arrows (reactions) in the metabolic map.

    Parameters
    ----------
    aes: Optional[Aesthetics]
        with accepted aesthetics being `{"reaction", "color", "size"}`.
    """

    def __init__(
        self, *, df: Optional[pd.DataFrame] = None, aes: Optional[Aesthetics] = None
    ):
        super().__init__(df, aes)
        self.mapping = {"color": "colors", "size": "sizes"}
        self.post_init()

    def check_type(self, data: pd.Series) -> pd.Series:
        """Check and validate the datatype."""
        if data.dtype == "O":
            LOGGER.warning("Geom data coerced distribution data to means.")
            data = data.apply(np.mean)
        assert (
            data.dtype.kind == "f" or data.dtype.kind == "i"
        ), "Data should be numbers"
        return data


class GeomMetabolite(GeomArrow):
    """Geometric mapping from aesthetics to the circles (metabolites) in the metabolic map.

    Parameters
    ----------
    aes: Optional[Aesthetics]
        with accepted aesthetics being `{"metabolite", "color", "size"}`.
    """

    def __init__(
        self, *, df: Optional[pd.DataFrame] = None, aes: Optional[Aesthetics] = None
    ):
        super().__init__(df=df, aes=aes)
        self.mapping = {"color": "met_colors", "size": "met_sizes"}
        self.post_init()


class GeomBoxPoint(GeomArrow):
    """Geometric mapping from aesthetics to the coloured boxes in the metabolic map.

    Parameters
    ----------
    aes: Optional[Aesthetics]
        with accepted aesthetics being `{"color"}`.
    side: str, default="right"
        Either "left", "right" or "hover". It determines the placement of the geom
        with respect to the reaction
    """

    def __init__(
        self,
        *,
        df: Optional[pd.DataFrame] = None,
        aes: Optional[Aesthetics] = None,
        side="right",
    ):
        super().__init__(df=df, aes=aes)
        self.mapping = {"color": "box_y" if side == "right" else "box_left_y"}
        self.post_init()
