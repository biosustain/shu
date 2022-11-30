"""Fixtures to aid tests."""

import pandas as pd
import pytest


@pytest.fixture
def df():
    """Provide dataframe for input."""
    return pd.DataFrame(
        {
            "r": ["a", "a", "b", "b", "c", "c", None, None, None, None],
            "flux": [1, 2, 3, 4, 6, 6, None, None, None, None],
            "kcat": [2, 4, 6, 7, 9, 10, None, None, None, None],
            "conc": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            "m": ["d", "e", "f", "g", "h", "d", "e", "f", "g", "h"],
        }
    )


@pytest.fixture
def df_cond():
    """Provide dataframe with conditions for input."""
    return pd.DataFrame(
        {
            "r": [
                "ACKr",
                "ACKr",
                "FTHFLi",
                "FTHFLi",
                "PTAr",
                "PTAr",
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
            "flux": [1, 2, 3, 4, 6, 6, None, None, None, None, None, None, None, None],
            "kcat": [2, 4, 6, 7, 9, 10, None, None, None, None, None, None, None, None],
            "km": [
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                1,
                2,
                3,
                4,
            ],
            "conc": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 1, None, None, None],
            "cond": ["x", "y", "x", "y", "x", "y", "x", "y", "x", "y", "", "", "", ""],
            "m": ["thf_c", "h2o_c", "glc_c", "methf_c", "accoa_c", "thf_c", "h2o_c", "glc_c", "methf_c", "accoa_c", "thf_c", "thf_c", "methf_c", "methf_c"],
        }
    )
