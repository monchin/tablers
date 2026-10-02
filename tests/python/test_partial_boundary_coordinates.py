"""Verify outer-frame repair preserves coordinates on every side of a table."""

import pytest
from tablers import Edge, TfSettings, find_all_cells_bboxes


def transform_bbox(
    bbox: tuple[float, float, float, float], side: str
) -> tuple[float, float, float, float]:
    """Reflect or transpose a right-edge example to exercise another outer side."""
    x1, y1, x2, y2 = bbox
    if side in ("left", "top"):
        x1, x2 = 100 - x2, 100 - x1
    if side in ("top", "bottom"):
        return y1, x1, y2, x2
    return x1, y1, x2, y2


def assert_repaired_grid(side: str, offset: float, missing: bool, enabled: bool) -> None:
    """Extract an explicit two-by-two grid and verify every repaired cell coordinate."""
    endpoint = (50.0 if missing else 100.0) + offset
    horizontal = [(0.0, y, endpoint, y) for y in (0.0, 50.0, 100.0)]
    vertical = [(x, 0.0, x, 100.0) for x in (0.0, 50.0)]
    if not missing:
        vertical.append((100.0, 0.0, 100.0, 50.0))
    edges = [transform_bbox(bbox, side) for bbox in horizontal + vertical]
    settings = TfSettings(
        horizontal_strategy="explicit",
        vertical_strategy="explicit",
        explicit_h_edges=[Edge("h", *bbox) for bbox in edges if bbox[1] == bbox[3]],
        explicit_v_edges=[Edge("v", *bbox) for bbox in edges if bbox[0] == bbox[2]],
        intersection_x_tolerance=3.0,
        intersection_y_tolerance=3.0,
        extend_partial_outer_boundaries=enabled,
    )
    outer = endpoint if missing else 100.0
    expected = sorted(
        transform_bbox((x1, y1, x2, y2), side)
        for x1, x2 in ((0.0, 50.0), (50.0, outer))
        for y1, y2 in ((0.0, 50.0), (50.0, 100.0))
    )
    cells = sorted(find_all_cells_bboxes(None, tf_settings=settings))
    assert len(cells) == 4
    for actual, wanted in zip(cells, expected, strict=True):
        assert actual == pytest.approx(wanted, abs=1e-5)
    assert len({(cell[0], cell[2]) for cell in cells}) == 2
    assert len({(cell[1], cell[3]) for cell in cells}) == 2


@pytest.mark.parametrize("side", ["left", "right", "top", "bottom"])
@pytest.mark.parametrize("offset", [0.1, 3.0], ids=["within-tolerance", "at-tolerance"])
def test_partial_boundary_keeps_existing_coordinate(side: str, offset: float) -> None:
    """Nearby endpoints must not create a second boundary and narrow extra cells."""
    assert_repaired_grid(side, offset, missing=False, enabled=True)


@pytest.mark.parametrize("side", ["left", "right", "top", "bottom"])
@pytest.mark.parametrize("enabled", [False, True], ids=["default", "partial-extension"])
def test_missing_boundary_beyond_tolerance_is_recovered(side: str, enabled: bool) -> None:
    """Both repair modes must retain the farther endpoint when an outer edge is absent."""
    assert_repaired_grid(side, 3.1, missing=True, enabled=enabled)
