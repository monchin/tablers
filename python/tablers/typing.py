"""
Public type aliases for the tablers library.

These types can be imported directly from ``tablers.typing`` for use in
type annotations, function signatures, and runtime validation.

Examples
--------
>>> from tablers.typing import BBox, Color, Point
>>> def my_func(bbox: BBox) -> None: ...
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Annotated, Literal, TypeAlias, TypedDict

if TYPE_CHECKING:
    from .tablers import Edge


def _validate_non_negative(value: int | float) -> bool:
    return value >= 0


NonNegativeFloat: TypeAlias = Annotated[float, _validate_non_negative]
"""A non-negative floating point number."""

NonNegativeInt: TypeAlias = Annotated[int, _validate_non_negative]
"""A non-negative integer."""

Point: TypeAlias = tuple[float, float]
"""A 2D point represented as (x, y) coordinates."""

BBox: TypeAlias = tuple[float, float, float, float]
"""A bounding box represented as (x1, y1, x2, y2) coordinates."""

Color: TypeAlias = tuple[int, int, int, int]
"""An RGBA color tuple, each component in range 0-255."""


class TfSettingItems(TypedDict, total=False):
    """
    TypedDict for TfSettings keyword arguments.

    All keys are optional. See TfSettings and the docs for defaults and semantics.
    """

    vertical_strategy: Literal["lines", "lines_strict", "text", "explicit"]
    horizontal_strategy: Literal["lines", "lines_strict", "text", "explicit"]
    snap_x_tolerance: float
    snap_y_tolerance: float
    join_x_tolerance: float
    join_y_tolerance: float
    edge_min_length: float
    edge_min_length_prefilter: float
    min_words_vertical: int
    min_words_horizontal: int
    intersection_x_tolerance: float
    intersection_y_tolerance: float
    include_single_cell: bool
    min_rows: int | None
    min_columns: int | None
    text_need_strip: bool
    text_x_tolerance: float
    text_y_tolerance: float
    text_keep_blank_chars: bool
    text_use_text_flow: bool
    text_read_in_clockwise: bool
    text_split_at_punctuation: Literal["all"] | str | None
    text_expand_ligatures: bool
    explicit_h_edges: list[Edge] | None
    explicit_v_edges: list[Edge] | None
    exclude_white_edges: bool


class WordsExtractSettingsItems(TypedDict, total=False):
    """
    TypedDict for WordsExtractSettings keyword arguments.

    All keys are optional. See WordsExtractSettings and the docs for defaults and semantics.
    """

    x_tolerance: float
    y_tolerance: float
    keep_blank_chars: bool
    use_text_flow: bool
    text_read_in_clockwise: bool
    split_at_punctuation: Literal["all"] | str | None
    expand_ligatures: bool
    need_strip: bool


__all__ = [
    "BBox",
    "Color",
    "NonNegativeFloat",
    "NonNegativeInt",
    "Point",
    "TfSettingItems",
    "WordsExtractSettingsItems",
]
