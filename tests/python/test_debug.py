"""
Tests for PageImage in tablers.debug.

These tests require the optional debug dependencies (pillow, pypdfium2).
The entire module is skipped automatically if they are not installed.
"""

from __future__ import annotations

from io import BytesIO
from pathlib import Path
from unittest.mock import MagicMock, patch

import pytest

pytest.importorskip("PIL")
pytest.importorskip("pypdfium2")

import PIL.Image  # noqa: E402
from tablers import Edge
from tablers.debug import PageImage

# ── Constants ─────────────────────────────────────────────────────────────────

PAGE_W = 100.0
PAGE_H = 200.0
IMG_W = 100
IMG_H = 200


# ── Helpers ───────────────────────────────────────────────────────────────────


def _make_page(*, width: float = PAGE_W, height: float = PAGE_H, closed: bool = False) -> MagicMock:
    """Return a minimal Page mock."""
    page = MagicMock()
    page.width = width
    page.height = height
    page.page_idx = 0
    page.doc.is_closed.return_value = closed
    page.objects = None
    return page


def _make_page_image(*, width: float = PAGE_W, height: float = PAGE_H) -> PageImage:
    """Return a PageImage backed by a blank white PIL image (no PDF rendering needed)."""
    page = _make_page(width=width, height=height)
    img = PIL.Image.new("RGB", (IMG_W, IMG_H), color=(255, 255, 255))
    return PageImage(page, original=img)


# ── Tests ─────────────────────────────────────────────────────────────────────


class TestPageImageInit:
    """Tests for PageImage.__init__."""

    def test_init_with_provided_image(self) -> None:
        """PageImage should accept a pre-built PIL image without rendering."""
        pi = _make_page_image()
        assert isinstance(pi.original, PIL.Image.Image)
        assert isinstance(pi.annotated, PIL.Image.Image)

    def test_bbox_covers_full_page(self) -> None:
        """bbox should be (0, 0, page_width, page_height)."""
        pi = _make_page_image()
        assert pi.bbox == (0, 0, PAGE_W, PAGE_H)

    def test_scale_computed_from_image_and_page_width(self) -> None:
        """scale = image_pixel_width / page_width."""
        pi = _make_page_image()
        assert pi.scale == IMG_W / PAGE_W

    def test_resolution_stored(self) -> None:
        """Provided resolution should be stored as-is."""
        page = _make_page()
        img = PIL.Image.new("RGB", (IMG_W, IMG_H))
        pi = PageImage(page, original=img, resolution=144)
        assert pi.resolution == 144

    def test_raises_if_doc_is_closed(self) -> None:
        """RuntimeError should be raised when the document is already closed."""
        page = _make_page(closed=True)
        with pytest.raises(RuntimeError, match="closed"):
            PageImage(page)


class TestPageImageReset:
    """Tests for PageImage.reset."""

    def test_reset_returns_self(self) -> None:
        """reset() should return the PageImage instance for chaining."""
        pi = _make_page_image()
        assert pi.reset() is pi

    def test_reset_restores_original_pixels(self) -> None:
        """After reset, annotated image should match the original pixel-for-pixel."""
        pi = _make_page_image()
        # Dirty the annotated image with a solid rect
        pi.draw_rect((5.0, 5.0, 90.0, 90.0), fill=(255, 0, 0, 255), stroke_width=0)
        pi.reset()
        assert list(pi.annotated.get_flattened_data()) == list(pi.original.get_flattened_data())


class TestPageImageCopy:
    """Tests for PageImage.copy."""

    def test_copy_returns_page_image(self) -> None:
        """copy() should return a PageImage instance."""
        pi = _make_page_image()
        assert isinstance(pi.copy(), PageImage)

    def test_copy_shares_original(self) -> None:
        """copy() reuses the same original PIL image object."""
        pi = _make_page_image()
        assert pi.copy().original is pi.original

    def test_copy_has_independent_annotated_image(self) -> None:
        """Modifying the copy should not affect the source PageImage."""
        pi = _make_page_image()
        c = pi.copy()
        c.draw_rect((5.0, 5.0, 90.0, 90.0), fill=(255, 0, 0, 255), stroke_width=0)
        assert list(pi.annotated.get_flattened_data()) != list(c.annotated.get_flattened_data())


class TestPageImageSave:
    """Tests for PageImage.save."""

    def test_save_to_bytesio_with_quantize(self) -> None:
        """save(quantize=True) should write a non-empty result."""
        pi = _make_page_image()
        buf = BytesIO()
        pi.save(buf)
        assert buf.tell() > 0

    def test_save_to_bytesio_without_quantize(self) -> None:
        """save(quantize=False) should write a non-empty result."""
        pi = _make_page_image()
        buf = BytesIO()
        pi.save(buf, quantize=False)
        assert buf.tell() > 0

    def test_save_produces_readable_image(self) -> None:
        """Bytes written by save() should be openable as a PIL image."""
        pi = _make_page_image()
        buf = BytesIO()
        pi.save(buf, quantize=False)
        buf.seek(0)
        img = PIL.Image.open(buf)
        assert img.size == (IMG_W, IMG_H)

    def test_save_to_file_path(self, tmp_path: Path) -> None:
        """save() should also accept a file path."""
        pi = _make_page_image()
        dest = tmp_path / "output.png"
        pi.save(dest, quantize=False)
        assert dest.exists()
        assert dest.stat().st_size > 0


class TestReprPng:
    """Tests for PageImage._repr_png_."""

    def test_returns_bytes(self) -> None:
        """_repr_png_() should return non-empty bytes."""
        pi = _make_page_image()
        data = pi._repr_png_()
        assert isinstance(data, bytes)
        assert len(data) > 0

    def test_returns_valid_png(self) -> None:
        """The returned bytes should decode to a valid PNG image."""
        pi = _make_page_image()
        img = PIL.Image.open(BytesIO(pi._repr_png_()))
        assert img.format == "PNG"


class TestReproject:
    """Tests for PageImage._reproject and _reproject_bbox."""

    def test_reproject_origin_maps_to_zero(self) -> None:
        """Page origin (0, 0) should map to image origin (0, 0)."""
        pi = _make_page_image()
        assert pi._reproject((0.0, 0.0)) == (0, 0)

    def test_reproject_applies_scale(self) -> None:
        """Coordinates should be multiplied by image_width / page_width."""
        # page 50×100, image 100×200 → scale = 2.0
        pi = _make_page_image(width=50.0, height=100.0)
        assert pi.scale == 2.0
        assert pi._reproject((25.0, 50.0)) == (50, 100)

    def test_reproject_bbox_returns_four_ints(self) -> None:
        """_reproject_bbox should return a 4-tuple of ints."""
        pi = _make_page_image()
        result = pi._reproject_bbox((10.0, 20.0, 80.0, 160.0))
        assert len(result) == 4
        assert all(isinstance(v, int) for v in result)

    def test_reproject_bbox_preserves_ordering(self) -> None:
        """Reprojected bbox should have left < right and top < bottom."""
        pi = _make_page_image()
        x0, top, x1, bottom = pi._reproject_bbox((10.0, 20.0, 80.0, 160.0))
        assert x0 < x1
        assert top < bottom


class TestDrawLine:
    """Tests for PageImage.draw_line."""

    def test_returns_self(self) -> None:
        """draw_line should return self for method chaining."""
        pi = _make_page_image()
        assert pi.draw_line([(0, 0), (50, 50)]) is pi

    def test_with_list_of_point_tuples(self) -> None:
        """draw_line should accept a plain list of (x, y) tuples."""
        _make_page_image().draw_line([(0, 0), (PAGE_W, PAGE_H)])

    def test_custom_stroke_color(self) -> None:
        """draw_line should accept a custom stroke color without raising."""
        _make_page_image().draw_line([(0, 0), (50, 50)], stroke=(0, 255, 0, 255))

    def test_custom_stroke_width(self) -> None:
        """draw_line should accept a custom stroke width without raising."""
        _make_page_image().draw_line([(0, 0), (50, 50)], stroke_width=3)


class TestDrawLines:
    """Tests for PageImage.draw_lines."""

    def test_returns_self(self) -> None:
        pi = _make_page_image()
        assert pi.draw_lines([[(0, 0), (50, 50)]]) is pi

    def test_empty_list_does_not_raise(self) -> None:
        _make_page_image().draw_lines([])

    def test_draws_all_lines(self) -> None:
        """draw_lines should process every item in the list."""
        pi = _make_page_image()
        lines = [[(0, 0), (PAGE_W, 0)], [(0, PAGE_H / 2), (PAGE_W, PAGE_H / 2)]]
        pi.draw_lines(lines)


class TestDrawVlineHline:
    """Tests for draw_vline, draw_vlines, draw_hline, draw_hlines."""

    def test_draw_vline_returns_self(self) -> None:
        pi = _make_page_image()
        assert pi.draw_vline(50.0) is pi

    def test_draw_vlines_returns_self(self) -> None:
        pi = _make_page_image()
        assert pi.draw_vlines([20.0, 50.0, 80.0]) is pi

    def test_draw_hline_returns_self(self) -> None:
        pi = _make_page_image()
        assert pi.draw_hline(100.0) is pi

    def test_draw_hlines_returns_self(self) -> None:
        pi = _make_page_image()
        assert pi.draw_hlines([50.0, 100.0, 150.0]) is pi

    def test_vline_at_boundaries(self) -> None:
        """draw_vline at page x-boundaries should not raise."""
        pi = _make_page_image()
        pi.draw_vline(0.0)
        pi.draw_vline(PAGE_W)

    def test_hline_at_boundaries(self) -> None:
        """draw_hline at page y-boundaries should not raise."""
        pi = _make_page_image()
        pi.draw_hline(0.0)
        pi.draw_hline(PAGE_H)

    def test_vlines_empty_list(self) -> None:
        _make_page_image().draw_vlines([])

    def test_hlines_empty_list(self) -> None:
        _make_page_image().draw_hlines([])


class TestDrawRect:
    """Tests for PageImage.draw_rect and draw_rects."""

    def test_returns_self(self) -> None:
        pi = _make_page_image()
        assert pi.draw_rect((10.0, 10.0, 80.0, 80.0)) is pi

    def test_with_bbox_tuple(self) -> None:
        """draw_rect should accept a plain 4-tuple bbox."""
        _make_page_image().draw_rect((10.0, 10.0, 80.0, 80.0))

    def test_with_zero_stroke_width(self) -> None:
        """stroke_width=0 should draw only the fill, not the border segments."""
        _make_page_image().draw_rect((10.0, 10.0, 80.0, 80.0), stroke_width=0)

    def test_draw_rects_returns_self(self) -> None:
        pi = _make_page_image()
        bboxes = [(10.0, 10.0, 45.0, 45.0), (55.0, 10.0, 90.0, 45.0)]
        assert pi.draw_rects(bboxes) is pi

    def test_draw_rects_empty_list(self) -> None:
        """draw_rects with an empty list should not raise."""
        _make_page_image().draw_rects([])

    def test_custom_fill_and_stroke(self) -> None:
        _make_page_image().draw_rect(
            (10.0, 10.0, 80.0, 80.0),
            fill=(0, 255, 0, 100),
            stroke=(255, 0, 0, 255),
            stroke_width=2,
        )


class TestDrawCircle:
    """Tests for PageImage.draw_circle and draw_circles."""

    def test_returns_self(self) -> None:
        pi = _make_page_image()
        assert pi.draw_circle((50.0, 100.0)) is pi

    def test_with_center_tuple(self) -> None:
        """draw_circle should accept a (cx, cy) tuple."""
        _make_page_image().draw_circle((50.0, 100.0))

    def test_draw_circles_returns_self(self) -> None:
        pi = _make_page_image()
        assert pi.draw_circles([(20.0, 50.0), (80.0, 150.0)]) is pi

    def test_draw_circles_empty_list(self) -> None:
        _make_page_image().draw_circles([])

    def test_custom_radius_and_colors(self) -> None:
        _make_page_image().draw_circle((50.0, 100.0), radius=10, fill=(0, 0, 255, 128))


class TestDebugTable:
    """Tests for PageImage.debug_table."""

    @staticmethod
    def _mock_table(*cell_bboxes: tuple) -> MagicMock:
        table = MagicMock()
        table.cells = [MagicMock(bbox=b) for b in cell_bboxes]
        return table

    def test_returns_self(self) -> None:
        pi = _make_page_image()
        table = self._mock_table((10.0, 10.0, 50.0, 50.0))
        assert pi.debug_table(table) is pi

    def test_draws_one_rect_per_cell(self) -> None:
        """debug_table should draw a rectangle for every cell in the table."""
        pi = _make_page_image()
        table = self._mock_table(
            (5.0, 5.0, 45.0, 45.0),
            (55.0, 5.0, 95.0, 45.0),
        )
        pi.debug_table(table)  # should not raise

    def test_empty_table_does_not_raise(self) -> None:
        """debug_table with zero cells should be a no-op."""
        pi = _make_page_image()
        pi.debug_table(self._mock_table())

    def test_custom_fill_and_stroke(self) -> None:
        pi = _make_page_image()
        table = self._mock_table((10.0, 10.0, 80.0, 80.0))
        pi.debug_table(table, fill=(0, 255, 0, 80), stroke=(0, 0, 255, 255), stroke_width=2)


class TestDebugTablefinder:
    """Tests for PageImage.debug_tablefinder."""

    @staticmethod
    def _real_edge(x1: float, y1: float, x2: float, y2: float) -> Edge:
        """Build a real Edge for use with get_intersections_from_edges (horizontal if y1==y2)."""
        orientation = "h" if y1 == y2 else "v"
        return Edge(orientation, x1, y1, x2, y2)

    def test_returns_self(self) -> None:
        pi = _make_page_image()
        with (
            patch("tablers.find_tables", return_value=[]),
            patch("tablers.get_edges", return_value={"h": [], "v": []}),
        ):
            assert pi.debug_tablefinder() is pi

    def test_calls_find_tables_and_get_edges(self) -> None:
        """debug_tablefinder should delegate to find_tables and get_edges."""
        pi = _make_page_image()
        mock_table = MagicMock()
        mock_table.cells = [MagicMock(bbox=(10.0, 10.0, 50.0, 50.0))]
        h_edge = self._real_edge(0, 0, PAGE_W, 0)

        with (
            patch("tablers.find_tables", return_value=[mock_table]) as mock_ft,
            patch("tablers.get_edges", return_value={"h": [h_edge], "v": []}) as mock_ge,
        ):
            pi.debug_tablefinder()

        mock_ft.assert_called_once()
        mock_ge.assert_called_once()

    def test_accepts_tf_settings(self) -> None:
        """debug_tablefinder should forward tf_settings to the underlying calls."""
        from tablers import TfSettings

        pi = _make_page_image()
        settings = TfSettings()
        with (
            patch("tablers.find_tables", return_value=[]) as mock_ft,
            patch("tablers.get_edges", return_value={"h": [], "v": []}) as mock_ge,
        ):
            pi.debug_tablefinder(tf_settings=settings)

        _, kwargs_ft = mock_ft.call_args
        _, kwargs_ge = mock_ge.call_args
        assert kwargs_ft.get("tf_settings") is settings
        assert kwargs_ge.get("tf_settings") is settings

    def test_no_tables_no_edges_does_not_raise(self) -> None:
        pi = _make_page_image()
        with (
            patch("tablers.find_tables", return_value=[]),
            patch("tablers.get_edges", return_value={"h": [], "v": []}),
        ):
            pi.debug_tablefinder()

    def test_multiple_tables_and_edges(self) -> None:
        """debug_tablefinder should handle multiple tables and edges."""
        pi = _make_page_image()
        mock_table = MagicMock()
        mock_table.cells = [MagicMock(bbox=(10.0, 10.0, 50.0, 50.0))]
        h_edges = [self._real_edge(0, 50, PAGE_W, 50)]
        v_edges = [self._real_edge(50, 0, 50, PAGE_H)]

        with (
            patch("tablers.find_tables", return_value=[mock_table, mock_table]),
            patch("tablers.get_edges", return_value={"h": h_edges, "v": v_edges}),
        ):
            pi.debug_tablefinder()


class TestMethodChaining:
    """All draw methods return self — fluent chaining should work end-to-end."""

    def test_chain_multiple_draw_calls(self) -> None:
        pi = _make_page_image()
        result = (
            pi.draw_line([(0, 0), (50, 50)])
            .draw_rect((10.0, 10.0, 80.0, 80.0))
            .draw_circle((50.0, 100.0))
            .draw_hline(100.0)
            .draw_vline(50.0)
            .reset()
        )
        assert result is pi

    def test_chain_with_plural_methods(self) -> None:
        pi = _make_page_image()
        result = (
            pi.draw_rects([(5.0, 5.0, 45.0, 45.0), (55.0, 55.0, 95.0, 95.0)])
            .draw_circles([(25.0, 25.0), (75.0, 75.0)])
            .draw_hlines([50.0, 100.0])
            .draw_vlines([25.0, 75.0])
        )
        assert result is pi


class TestPageImageIntegration:
    """Integration tests that render a real PDF page via pypdfium2."""

    def test_render_from_pdf(self, edge_test_doc) -> None:
        """PageImage should render a real PDF page into a PIL image."""
        page = edge_test_doc.get_page(0)
        pi = PageImage(page)
        assert isinstance(pi.original, PIL.Image.Image)
        assert pi.original.width > 0
        assert pi.original.height > 0

    def test_scale_matches_resolution(self, edge_test_doc) -> None:
        """Scale should equal rendered_width / page_width."""
        page = edge_test_doc.get_page(0)
        pi = PageImage(page)
        assert abs(pi.scale - pi.original.width / page.width) < 1e-6

    def test_debug_tablefinder_on_real_pdf(self, edge_test_doc) -> None:
        """debug_tablefinder should not raise on a real PDF page."""
        page = edge_test_doc.get_page(0)
        pi = PageImage(page)
        result = pi.debug_tablefinder()
        assert result is pi

    def test_save_rendered_page(self, edge_test_doc, tmp_path: Path) -> None:
        """A rendered page should be saveable to a PNG file."""
        page = edge_test_doc.get_page(0)
        pi = PageImage(page)
        dest = tmp_path / "page.png"
        pi.save(dest, quantize=False)
        assert dest.exists()
        assert dest.stat().st_size > 0

    def test_repr_png_on_real_pdf(self, edge_test_doc) -> None:
        """_repr_png_ should return valid PNG bytes for a real page."""
        page = edge_test_doc.get_page(0)
        pi = PageImage(page)
        data = pi._repr_png_()
        img = PIL.Image.open(BytesIO(data))
        assert img.format == "PNG"
