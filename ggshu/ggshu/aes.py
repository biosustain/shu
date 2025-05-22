"""Aesthetics, map from df variables to grammar variables."""

from typing import Dict, Optional

Aesthetics = Dict[str, str]


def aes(
    reaction: Optional[str] = None,
    metabolite: Optional[str] = None,
    condition: Optional[str] = None,
    y: Optional[str] = None,
    color: Optional[str] = None,
    size: Optional[str] = None,
    stack: Optional[str] = None,
    ymin: Optional[str] = None,
    ymax: Optional[str] = None,
) -> Aesthetics:
    """Map from dataframe variables to grammar graphics variables."""
    # instead of using **kwargs, we specify the exact accepted aes
    # so that the users get notified by the LSP (or a runtime error)
    # if something is wrong
    aesthetics = {
        "reaction": reaction,
        "metabolite": metabolite,
        "condition": condition,
        "y": y,
        "ymin": ymin,
        "ymax": ymax,
        "color": color,
        "size": size,
        "stack": stack,
    }
    return {k: v for k, v in aesthetics.items() if v is not None}
