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

from typing import Annotated, TypeAlias


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

__all__ = [
    "BBox",
    "Color",
    "NonNegativeFloat",
    "NonNegativeInt",
    "Point",
]
