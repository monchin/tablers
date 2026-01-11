import math
from typing import Any, cast

from .tablers import Edge


def plumber_edge_to_tablers_edge(
    plumber_edge: dict[str, Any], page_rotation: float, page_height: float, page_width: float
) -> Edge:
    orientation = plumber_edge["orientation"]
    x1 = cast(float, plumber_edge["x0"])
    y1 = cast(float, plumber_edge["y0"])
    x2 = cast(float, plumber_edge["x1"])
    y2 = cast(float, plumber_edge["y1"])
    if math.isclose(page_rotation, 0.0, abs_tol=1e-3) or math.isclose(
        page_rotation, 180.0, abs_tol=1e-3
    ):
        y1 = page_height - y1
        y2 = page_height - y2
    else:
        x1 = page_width - x1
        x2 = page_width - x2

    width = cast(float, plumber_edge["linewidth"])
    color = cast(tuple[int, int, int], plumber_edge["stroking_color"])
    return Edge(
        orientation=orientation,
        x1=x1,
        y1=y1,
        x2=x2,
        y2=y2,
        width=width,
        color=(color[0], color[1], color[2], 255),
    )
