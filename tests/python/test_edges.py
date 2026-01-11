"""
Tests for edge functions.
"""

import pytest
from tablers import Document, get_edges


class TestGetEdges:
    """Tests for get_edges function."""

    def test_get_edges_from_pdf(self, multiple_move_to_in_one_seg_doc: Document) -> None:
        """Test extracting edges from multiple-move-to-in-one-seg.pdf."""
        page = multiple_move_to_in_one_seg_doc.get_page(0)
        page.extract_objects()

        edges = get_edges(page)

        # Verify structure
        assert "h" in edges
        assert "v" in edges

        # Verify horizontal edges
        h_edges = edges["h"]
        assert len(h_edges) == 5

        # Check first horizontal edge
        assert h_edges[0].orientation == "h"
        assert h_edges[0].x1 == pytest.approx(90, abs=0.01)
        assert h_edges[0].y1 == pytest.approx(72.23999, abs=0.01)
        assert h_edges[0].x2 == pytest.approx(504.84, abs=0.01)
        assert h_edges[0].y2 == pytest.approx(72.23999, abs=0.01)
        assert h_edges[0].width == pytest.approx(0.47998047, abs=0.001)
        assert h_edges[0].color == (0, 0, 0, 255)

        # Check second horizontal edge (starts at different x1)
        assert h_edges[1].x1 == pytest.approx(297.36002, abs=0.01)
        assert h_edges[1].y1 == pytest.approx(88.32001, abs=0.01)

        # Check remaining horizontal edges
        assert h_edges[2].y1 == pytest.approx(104.400024, abs=0.01)
        assert h_edges[3].y1 == pytest.approx(120.47998, abs=0.01)
        assert h_edges[4].y1 == pytest.approx(136.68, abs=0.01)

        # Verify vertical edges
        v_edges = edges["v"]
        assert len(v_edges) == 3

        # Check vertical edges
        assert v_edges[0].orientation == "v"
        assert v_edges[0].x1 == pytest.approx(90.24, abs=0.01)
        assert v_edges[0].y1 == pytest.approx(72.47998, abs=0.01)
        assert v_edges[0].y2 == pytest.approx(136.91998, abs=0.01)
        assert v_edges[0].color == (0, 0, 0, 255)

        assert v_edges[1].x1 == pytest.approx(297.6, abs=0.01)
        assert v_edges[2].x1 == pytest.approx(505.08002, abs=0.01)
