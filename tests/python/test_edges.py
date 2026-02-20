"""
Tests for edge functions.
"""

from typing import TYPE_CHECKING

import pytest
from tablers import Document, Edge, get_edges
from tablers.edges import plumber_edge_to_tablers_edge

if TYPE_CHECKING:
    from tablers import Document


class TestPlumberEdgeToTablersEdge:
    """Tests for plumber_edge_to_tablers_edge function."""

    def test_horizontal_edge_rotation_0(self) -> None:
        """Test horizontal edge conversion with rotation 0."""
        plumber_edge = {
            "orientation": "h",
            "x0": 10.0,
            "y0": 20.0,
            "x1": 100.0,
            "y1": 20.0,
            "linewidth": 1.5,
            "stroking_color": (0, 0, 0),
        }
        page_height = 800.0
        page_width = 600.0

        edge = plumber_edge_to_tablers_edge(plumber_edge, 0.0, page_height, page_width)

        assert edge.orientation == "h"
        assert edge.x1 == 10.0
        assert edge.y1 == page_height - 20.0  # Y is flipped
        assert edge.x2 == 100.0
        assert edge.y2 == page_height - 20.0
        assert edge.width == 1.5
        assert edge.color == (0, 0, 0, 255)

    def test_vertical_edge_rotation_0(self) -> None:
        """Test vertical edge conversion with rotation 0."""
        plumber_edge = {
            "orientation": "v",
            "x0": 50.0,
            "y0": 10.0,
            "x1": 50.0,
            "y1": 200.0,
            "linewidth": 2.0,
            "stroking_color": (255, 0, 0),
        }
        page_height = 800.0
        page_width = 600.0

        edge = plumber_edge_to_tablers_edge(plumber_edge, 0.0, page_height, page_width)

        assert edge.orientation == "v"
        assert edge.x1 == 50.0
        assert edge.y1 == page_height - 10.0
        assert edge.x2 == 50.0
        assert edge.y2 == page_height - 200.0
        assert edge.width == 2.0
        assert edge.color == (255, 0, 0, 255)

    def test_horizontal_edge_rotation_180(self) -> None:
        """Test horizontal edge conversion with rotation 180."""
        plumber_edge = {
            "orientation": "h",
            "x0": 10.0,
            "y0": 20.0,
            "x1": 100.0,
            "y1": 20.0,
            "linewidth": 1.0,
            "stroking_color": (0, 255, 0),
        }
        page_height = 800.0
        page_width = 600.0

        edge = plumber_edge_to_tablers_edge(plumber_edge, 180.0, page_height, page_width)

        # Rotation 180 should still flip Y coordinates
        assert edge.orientation == "h"
        assert edge.y1 == page_height - 20.0
        assert edge.y2 == page_height - 20.0

    def test_edge_rotation_90(self) -> None:
        """Test edge conversion with rotation 90 (landscape)."""
        plumber_edge = {
            "orientation": "h",
            "x0": 10.0,
            "y0": 20.0,
            "x1": 100.0,
            "y1": 20.0,
            "linewidth": 1.0,
            "stroking_color": (0, 0, 255),
        }
        page_height = 800.0
        page_width = 600.0

        edge = plumber_edge_to_tablers_edge(plumber_edge, 90.0, page_height, page_width)

        # Rotation 90 should flip X coordinates using page_width
        assert edge.x1 == page_width - 10.0
        assert edge.x2 == page_width - 100.0

    def test_edge_rotation_270(self) -> None:
        """Test edge conversion with rotation 270."""
        plumber_edge = {
            "orientation": "v",
            "x0": 30.0,
            "y0": 40.0,
            "x1": 30.0,
            "y1": 150.0,
            "linewidth": 0.5,
            "stroking_color": (128, 128, 128),
        }
        page_height = 800.0
        page_width = 600.0

        edge = plumber_edge_to_tablers_edge(plumber_edge, 270.0, page_height, page_width)

        # Rotation 270 should flip X coordinates using page_width
        assert edge.x1 == page_width - 30.0
        assert edge.x2 == page_width - 30.0


class TestEdgeFromPython:
    """Tests for Edge creation directly from Python."""

    def test_create_horizontal_edge(self) -> None:
        """Test creating a horizontal edge from Python."""
        edge = Edge("h", 0.0, 10.0, 100.0, 10.0)

        assert edge.orientation == "h"
        assert edge.x1 == 0.0
        assert edge.y1 == 10.0
        assert edge.x2 == 100.0
        assert edge.y2 == 10.0
        assert edge.width == 1.0  # default
        assert edge.color == (0, 0, 0, 255)  # default

    def test_create_vertical_edge(self) -> None:
        """Test creating a vertical edge from Python."""
        edge = Edge("v", 50.0, 0.0, 50.0, 200.0)

        assert edge.orientation == "v"
        assert edge.x1 == 50.0
        assert edge.y1 == 0.0
        assert edge.x2 == 50.0
        assert edge.y2 == 200.0

    def test_create_edge_with_width(self) -> None:
        """Test creating an edge with custom width."""
        edge = Edge("h", 0.0, 10.0, 100.0, 10.0, width=2.5)

        assert edge.width == 2.5

    def test_create_edge_with_color(self) -> None:
        """Test creating an edge with custom color."""
        edge = Edge("h", 0.0, 10.0, 100.0, 10.0, color=(255, 0, 0, 128))

        assert edge.color == (255, 0, 0, 128)

    def test_create_edge_with_all_params(self) -> None:
        """Test creating an edge with all parameters."""
        edge = Edge("v", 25.0, 50.0, 25.0, 150.0, width=3.0, color=(0, 128, 255, 200))

        assert edge.orientation == "v"
        assert edge.x1 == 25.0
        assert edge.y1 == 50.0
        assert edge.x2 == 25.0
        assert edge.y2 == 150.0
        assert edge.width == 3.0
        assert edge.color == (0, 128, 255, 200)

    def test_edge_repr(self) -> None:
        """Test Edge __repr__ method."""
        edge = Edge("h", 10.0, 20.0, 100.0, 20.0, width=1.5, color=(255, 128, 0, 255))
        repr_str = repr(edge)

        assert "Edge" in repr_str
        assert "h" in repr_str
        assert "10" in repr_str
        assert "20" in repr_str
        assert "100" in repr_str
        assert "1.5" in repr_str

    def test_edge_equality(self) -> None:
        """Test Edge __eq__ method."""
        edge1 = Edge("h", 10.0, 20.0, 100.0, 20.0)
        edge2 = Edge("h", 10.0, 20.0, 100.0, 20.0)
        edge3 = Edge("h", 10.0, 20.0, 100.0, 30.0)  # different y2

        assert edge1 == edge2
        assert edge1 != edge3

    def test_edge_equality_ignores_width_and_color(self) -> None:
        """Test that Edge equality only considers coordinates."""
        edge1 = Edge("h", 10.0, 20.0, 100.0, 20.0, width=1.0, color=(0, 0, 0, 255))
        edge2 = Edge("h", 10.0, 20.0, 100.0, 20.0, width=5.0, color=(255, 255, 255, 0))

        # Should be equal because coordinates are the same
        assert edge1 == edge2


class TestConvertedEdgeUsableInRust:
    """Tests to verify converted edges work correctly with Rust internals."""

    def test_converted_edge_has_correct_properties(self) -> None:
        """Test that converted edge has all expected properties."""
        plumber_edge = {
            "orientation": "h",
            "x0": 0.0,
            "y0": 100.0,
            "x1": 500.0,
            "y1": 100.0,
            "linewidth": 1.0,
            "stroking_color": (0, 0, 0),
        }

        edge = plumber_edge_to_tablers_edge(plumber_edge, 0.0, 800.0, 600.0)

        # Verify all properties are accessible (this ensures Rust internals work)
        assert isinstance(edge.orientation, str)
        assert isinstance(edge.x1, float)
        assert isinstance(edge.y1, float)
        assert isinstance(edge.x2, float)
        assert isinstance(edge.y2, float)
        assert isinstance(edge.width, float)
        assert isinstance(edge.color, tuple)
        assert len(edge.color) == 4

    def test_converted_edge_repr_works(self) -> None:
        """Test that __repr__ works on converted edge."""
        plumber_edge = {
            "orientation": "v",
            "x0": 50.0,
            "y0": 0.0,
            "x1": 50.0,
            "y1": 400.0,
            "linewidth": 2.0,
            "stroking_color": (128, 64, 32),
        }

        edge = plumber_edge_to_tablers_edge(plumber_edge, 0.0, 800.0, 600.0)
        repr_str = repr(edge)

        assert "Edge" in repr_str
        assert "v" in repr_str

    def test_converted_edge_equality_with_python_created(self) -> None:
        """Test that converted edge can be compared with Python-created edge."""
        plumber_edge = {
            "orientation": "h",
            "x0": 10.0,
            "y0": 20.0,
            "x1": 100.0,
            "y1": 20.0,
            "linewidth": 1.0,
            "stroking_color": (0, 0, 0),
        }
        page_height = 800.0

        converted = plumber_edge_to_tablers_edge(plumber_edge, 0.0, page_height, 600.0)
        # Create equivalent edge from Python
        python_edge = Edge(
            "h",
            10.0,
            page_height - 20.0,
            100.0,
            page_height - 20.0,
            width=1.0,
            color=(0, 0, 0, 255),
        )

        assert converted == python_edge

    def test_multiple_edges_can_be_created(self) -> None:
        """Test that multiple edges can be created and stored."""
        edges = []
        for i in range(10):
            edge = Edge("h", float(i * 10), 0.0, float(i * 10 + 50), 0.0)
            edges.append(edge)

        assert len(edges) == 10
        for i, edge in enumerate(edges):
            assert edge.x1 == float(i * 10)
            assert edge.x2 == float(i * 10 + 50)

    def test_edge_with_invalid_orientation_raises(self) -> None:
        """Test that invalid orientation raises an error."""
        with pytest.raises(ValueError, match="Invalid orientation"):
            Edge("x", 0.0, 0.0, 100.0, 0.0)  # Invalid orientation


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


class TestExplicitEdgesTableFinding:
    """Tests for table finding using explicit edges."""

    def test_find_tables_with_explicit_edges_simple_grid(self, edge_test_doc: "Document") -> None:
        """Test finding tables using explicit edges to create a simple 2x2 grid."""
        from tablers import TfSettings, find_tables

        page = edge_test_doc.get_page(0)

        # Create a 2x2 grid using explicit edges
        # Horizontal edges at y=700, y=750, y=800
        h_edges = [
            Edge("h", 50.0, 700.0, 150.0, 700.0),
            Edge("h", 50.0, 750.0, 150.0, 750.0),
            Edge("h", 50.0, 800.0, 150.0, 800.0),
        ]
        # Vertical edges at x=50, x=100, x=150
        v_edges = [
            Edge("v", 50.0, 700.0, 50.0, 800.0),
            Edge("v", 100.0, 700.0, 100.0, 800.0),
            Edge("v", 150.0, 700.0, 150.0, 800.0),
        ]

        settings = TfSettings(
            horizontal_strategy="explicit",
            vertical_strategy="explicit",
            explicit_h_edges=h_edges,
            explicit_v_edges=v_edges,
        )

        tables = find_tables(page, extract_text=False, tf_settings=settings)

        assert len(tables) == 1
        table = tables[0]
        # 2x2 grid should have 4 cells
        assert len(table.cells) == 4

    def test_find_tables_with_explicit_edges_single_cell(self, edge_test_doc: "Document") -> None:
        """Test finding a single cell table using explicit edges."""
        from tablers import TfSettings, find_tables

        page = edge_test_doc.get_page(0)

        # Create a single cell
        h_edges = [
            Edge("h", 0.0, 0.0, 100.0, 0.0),
            Edge("h", 0.0, 100.0, 100.0, 100.0),
        ]
        v_edges = [
            Edge("v", 0.0, 0.0, 0.0, 100.0),
            Edge("v", 100.0, 0.0, 100.0, 100.0),
        ]

        settings = TfSettings(
            horizontal_strategy="explicit",
            vertical_strategy="explicit",
            explicit_h_edges=h_edges,
            explicit_v_edges=v_edges,
            include_single_cell=True,
        )

        tables = find_tables(page, extract_text=False, tf_settings=settings)

        assert len(tables) == 1
        assert len(tables[0].cells) == 1

    def test_find_tables_explicit_with_no_edges_returns_empty(
        self, edge_test_doc: "Document"
    ) -> None:
        """Test that explicit strategy with no edges returns no tables."""
        from tablers import TfSettings, find_tables

        page = edge_test_doc.get_page(0)

        settings = TfSettings(
            horizontal_strategy="explicit",
            vertical_strategy="explicit",
            explicit_h_edges=None,
            explicit_v_edges=None,
        )

        tables = find_tables(page, extract_text=False, tf_settings=settings)

        assert len(tables) == 0

    def test_find_tables_explicit_edges_with_text_extraction(
        self, edge_test_doc: "Document"
    ) -> None:
        """Test that text extraction works with explicit edges."""
        from tablers import TfSettings, find_tables

        page = edge_test_doc.get_page(0)

        # Create a grid that should cover some text in the PDF
        h_edges = [
            Edge("h", 50.0, 700.0, 200.0, 700.0),
            Edge("h", 50.0, 750.0, 200.0, 750.0),
        ]
        v_edges = [
            Edge("v", 50.0, 700.0, 50.0, 750.0),
            Edge("v", 200.0, 700.0, 200.0, 750.0),
        ]

        settings = TfSettings(
            horizontal_strategy="explicit",
            vertical_strategy="explicit",
            explicit_h_edges=h_edges,
            explicit_v_edges=v_edges,
            include_single_cell=True,
        )

        tables = find_tables(page, extract_text=True, tf_settings=settings)

        assert len(tables) == 1
        # Verify text extraction worked (cells should have text attribute)
        for cell in tables[0].cells:
            assert hasattr(cell, "text")
            assert isinstance(cell.text, str)

    def test_explicit_edges_mixed_with_lines_strategy(self, edge_test_doc: "Document") -> None:
        """Test using explicit edges for one direction and lines for another."""
        from tablers import TfSettings, find_tables

        page = edge_test_doc.get_page(0)

        # Only provide explicit horizontal edges
        h_edges = [
            Edge("h", 0.0, 500.0, 500.0, 500.0),
            Edge("h", 0.0, 600.0, 500.0, 600.0),
        ]

        settings = TfSettings(
            horizontal_strategy="explicit",
            vertical_strategy="lines_strict",  # Use lines from PDF for vertical
            explicit_h_edges=h_edges,
            explicit_v_edges=None,
        )

        # This should still work, combining explicit h-edges with detected v-edges
        tables = find_tables(page, extract_text=False, tf_settings=settings)
        assert isinstance(tables, list)

    def test_get_edges_with_explicit_edges(self, edge_test_doc: "Document") -> None:
        """Test get_edges function returns explicit edges correctly."""
        from tablers import get_edges

        page = edge_test_doc.get_page(0)

        h_edge = Edge("h", 10.0, 20.0, 100.0, 20.0)
        v_edge = Edge("v", 50.0, 0.0, 50.0, 100.0)

        edges = get_edges(
            page,
            None,
            **{
                "horizontal_strategy": "explicit",
                "vertical_strategy": "explicit",
                "explicit_h_edges": [h_edge],
                "explicit_v_edges": [v_edge],
            },
        )

        assert "h" in edges
        assert "v" in edges
        # Should contain at least our explicit edges
        assert len(edges["h"]) >= 1
        assert len(edges["v"]) >= 1

    def test_find_all_cells_bboxes_with_explicit_edges(self, edge_test_doc: "Document") -> None:
        """Test find_all_cells_bboxes with explicit edges."""
        from tablers import TfSettings, find_all_cells_bboxes

        page = edge_test_doc.get_page(0)

        # Create a 2x2 grid
        h_edges = [
            Edge("h", 0.0, 0.0, 100.0, 0.0),
            Edge("h", 0.0, 50.0, 100.0, 50.0),
            Edge("h", 0.0, 100.0, 100.0, 100.0),
        ]
        v_edges = [
            Edge("v", 0.0, 0.0, 0.0, 100.0),
            Edge("v", 50.0, 0.0, 50.0, 100.0),
            Edge("v", 100.0, 0.0, 100.0, 100.0),
        ]

        settings = TfSettings(
            horizontal_strategy="explicit",
            vertical_strategy="explicit",
            explicit_h_edges=h_edges,
            explicit_v_edges=v_edges,
        )

        cells = find_all_cells_bboxes(page, tf_settings=settings)

        # 2x2 grid should produce 4 cells
        assert len(cells) == 4
        # Each cell should be a tuple of 4 floats (bbox)
        for cell in cells:
            assert isinstance(cell, tuple)
            assert len(cell) == 4


class TestFindTablesWithoutPage:
    """Tests for find_tables with page=None when using explicit edges."""

    def test_find_tables_without_page_explicit_strategy(self) -> None:
        """Test find_tables with explicit edges and no page."""
        from tablers import TfSettings, find_tables

        # Create a 2x2 grid using explicit edges
        h_edges = [
            Edge("h", 0.0, 0.0, 100.0, 0.0),
            Edge("h", 0.0, 50.0, 100.0, 50.0),
            Edge("h", 0.0, 100.0, 100.0, 100.0),
        ]
        v_edges = [
            Edge("v", 0.0, 0.0, 0.0, 100.0),
            Edge("v", 50.0, 0.0, 50.0, 100.0),
            Edge("v", 100.0, 0.0, 100.0, 100.0),
        ]

        settings = TfSettings(
            horizontal_strategy="explicit",
            vertical_strategy="explicit",
            explicit_h_edges=h_edges,
            explicit_v_edges=v_edges,
        )

        # Call with page=None and extract_text=False
        tables = find_tables(page=None, extract_text=False, tf_settings=settings)

        # 2x2 grid should produce 1 table with 4 cells
        assert len(tables) == 1
        assert len(tables[0].cells) == 4

    def test_find_tables_without_page_single_cell(self) -> None:
        """Test find_tables with a single cell and no page."""
        from tablers import TfSettings, find_tables

        # Create a single cell
        h_edges = [
            Edge("h", 0.0, 0.0, 100.0, 0.0),
            Edge("h", 0.0, 100.0, 100.0, 100.0),
        ]
        v_edges = [
            Edge("v", 0.0, 0.0, 0.0, 100.0),
            Edge("v", 100.0, 0.0, 100.0, 100.0),
        ]

        settings = TfSettings(
            horizontal_strategy="explicit",
            vertical_strategy="explicit",
            explicit_h_edges=h_edges,
            explicit_v_edges=v_edges,
            include_single_cell=True,
        )

        tables = find_tables(page=None, extract_text=False, tf_settings=settings)

        assert len(tables) == 1
        assert len(tables[0].cells) == 1

    def test_find_tables_without_page_empty_edges(self) -> None:
        """Test find_tables with empty explicit edges returns no tables."""
        from tablers import TfSettings, find_tables

        settings = TfSettings(
            horizontal_strategy="explicit",
            vertical_strategy="explicit",
            explicit_h_edges=[],
            explicit_v_edges=[],
        )

        tables = find_tables(page=None, extract_text=False, tf_settings=settings)

        assert len(tables) == 0

    def test_find_tables_without_page_extract_text_raises(self) -> None:
        """Test that find_tables raises error when page is None and extract_text is True."""
        from tablers import TfSettings, find_tables

        h_edges = [
            Edge("h", 0.0, 0.0, 100.0, 0.0),
            Edge("h", 0.0, 100.0, 100.0, 100.0),
        ]
        v_edges = [
            Edge("v", 0.0, 0.0, 0.0, 100.0),
            Edge("v", 100.0, 0.0, 100.0, 100.0),
        ]

        settings = TfSettings(
            horizontal_strategy="explicit",
            vertical_strategy="explicit",
            explicit_h_edges=h_edges,
            explicit_v_edges=v_edges,
        )

        with pytest.raises(ValueError, match="page must be provided when extract_text is true"):
            find_tables(page=None, extract_text=True, tf_settings=settings)

    def test_find_tables_without_page_non_explicit_raises(self) -> None:
        """Test that find_tables raises error when page is None and strategy is not explicit."""
        from tablers import TfSettings, find_tables

        settings = TfSettings(
            horizontal_strategy="lines",  # Not explicit
            vertical_strategy="explicit",
        )

        with pytest.raises(ValueError, match="page can only be None"):
            find_tables(page=None, extract_text=False, tf_settings=settings)

    def test_find_tables_without_page_3x3_grid(self) -> None:
        """Test find_tables with a 3x3 grid and no page."""
        from tablers import TfSettings, find_tables

        # Create a 3x3 grid
        h_edges = [
            Edge("h", 0.0, 0.0, 150.0, 0.0),
            Edge("h", 0.0, 50.0, 150.0, 50.0),
            Edge("h", 0.0, 100.0, 150.0, 100.0),
            Edge("h", 0.0, 150.0, 150.0, 150.0),
        ]
        v_edges = [
            Edge("v", 0.0, 0.0, 0.0, 150.0),
            Edge("v", 50.0, 0.0, 50.0, 150.0),
            Edge("v", 100.0, 0.0, 100.0, 150.0),
            Edge("v", 150.0, 0.0, 150.0, 150.0),
        ]

        settings = TfSettings(
            horizontal_strategy="explicit",
            vertical_strategy="explicit",
            explicit_h_edges=h_edges,
            explicit_v_edges=v_edges,
        )

        tables = find_tables(page=None, extract_text=False, tf_settings=settings)

        assert len(tables) == 1
        # 3x3 grid should have 9 cells
        assert len(tables[0].cells) == 9

    def test_find_tables_without_page_verifies_cell_bboxes(self) -> None:
        """Test that cell bboxes are correct when using explicit edges without page."""
        from tablers import TfSettings, find_tables

        # Create a simple 1x2 grid (2 cells in a row)
        h_edges = [
            Edge("h", 0.0, 0.0, 200.0, 0.0),
            Edge("h", 0.0, 100.0, 200.0, 100.0),
        ]
        v_edges = [
            Edge("v", 0.0, 0.0, 0.0, 100.0),
            Edge("v", 100.0, 0.0, 100.0, 100.0),
            Edge("v", 200.0, 0.0, 200.0, 100.0),
        ]

        settings = TfSettings(
            horizontal_strategy="explicit",
            vertical_strategy="explicit",
            explicit_h_edges=h_edges,
            explicit_v_edges=v_edges,
        )

        tables = find_tables(page=None, extract_text=False, tf_settings=settings)

        assert len(tables) == 1
        assert len(tables[0].cells) == 2

        # Sort cells by x1 to ensure consistent ordering
        cells = sorted(tables[0].cells, key=lambda c: c.bbox[0])

        # First cell should be (0, 0, 100, 100)
        assert cells[0].bbox[0] == pytest.approx(0.0)
        assert cells[0].bbox[1] == pytest.approx(0.0)
        assert cells[0].bbox[2] == pytest.approx(100.0)
        assert cells[0].bbox[3] == pytest.approx(100.0)

        # Second cell should be (100, 0, 200, 100)
        assert cells[1].bbox[0] == pytest.approx(100.0)
        assert cells[1].bbox[1] == pytest.approx(0.0)
        assert cells[1].bbox[2] == pytest.approx(200.0)
        assert cells[1].bbox[3] == pytest.approx(100.0)


class TestExplicitEdgesWithoutPage:
    """Tests for using explicit edges without a PDF page."""

    def test_find_all_cells_bboxes_without_page(self) -> None:
        """Test find_all_cells_bboxes with explicit edges and no page."""
        from tablers import TfSettings, find_all_cells_bboxes

        # Create a 2x2 grid using explicit edges
        h_edges = [
            Edge("h", 0.0, 0.0, 100.0, 0.0),
            Edge("h", 0.0, 50.0, 100.0, 50.0),
            Edge("h", 0.0, 100.0, 100.0, 100.0),
        ]
        v_edges = [
            Edge("v", 0.0, 0.0, 0.0, 100.0),
            Edge("v", 50.0, 0.0, 50.0, 100.0),
            Edge("v", 100.0, 0.0, 100.0, 100.0),
        ]

        settings = TfSettings(
            horizontal_strategy="explicit",
            vertical_strategy="explicit",
            explicit_h_edges=h_edges,
            explicit_v_edges=v_edges,
        )

        # Call without page (page=None)
        cells = find_all_cells_bboxes(None, tf_settings=settings)

        # 2x2 grid should produce 4 cells
        assert len(cells) == 4
        for cell in cells:
            assert isinstance(cell, tuple)
            assert len(cell) == 4

    def test_get_edges_without_page(self) -> None:
        """Test get_edges with explicit edges and no page."""
        h_edge = Edge("h", 10.0, 20.0, 100.0, 20.0)
        v_edge = Edge("v", 50.0, 0.0, 50.0, 100.0)

        edges = get_edges(
            None,
            None,
            horizontal_strategy="explicit",
            vertical_strategy="explicit",
            explicit_h_edges=[h_edge],
            explicit_v_edges=[v_edge],
        )

        assert "h" in edges
        assert "v" in edges
        assert len(edges["h"]) == 1
        assert len(edges["v"]) == 1

    def test_find_all_cells_bboxes_without_page_non_explicit_raises(self) -> None:
        """
        Test that find_all_cells_bboxes raises error when page is None and strategy is not explicit.
        """
        from tablers import TfSettings, find_all_cells_bboxes

        settings = TfSettings(
            horizontal_strategy="lines",  # Not explicit
            vertical_strategy="explicit",
        )

        with pytest.raises(RuntimeError, match="page can only be None"):
            find_all_cells_bboxes(None, tf_settings=settings)

    def test_get_edges_without_page_non_explicit_raises(self) -> None:
        """Test that get_edges raises error when page is None and strategy is not explicit."""
        with pytest.raises(RuntimeError, match="page can only be None"):
            get_edges(
                None,
                None,
                horizontal_strategy="explicit",
                vertical_strategy="lines_strict",  # Not explicit
            )

    def test_find_all_cells_bboxes_without_page_single_cell(self) -> None:
        """Test creating a single cell without a page."""
        from tablers import TfSettings, find_all_cells_bboxes

        # Create a single cell
        h_edges = [
            Edge("h", 0.0, 0.0, 100.0, 0.0),
            Edge("h", 0.0, 100.0, 100.0, 100.0),
        ]
        v_edges = [
            Edge("v", 0.0, 0.0, 0.0, 100.0),
            Edge("v", 100.0, 0.0, 100.0, 100.0),
        ]

        settings = TfSettings(
            horizontal_strategy="explicit",
            vertical_strategy="explicit",
            explicit_h_edges=h_edges,
            explicit_v_edges=v_edges,
        )

        cells = find_all_cells_bboxes(None, tf_settings=settings)

        assert len(cells) == 1
        # Verify the cell bbox
        cell = cells[0]
        assert cell[0] == pytest.approx(0.0)  # x1
        assert cell[1] == pytest.approx(0.0)  # y1
        assert cell[2] == pytest.approx(100.0)  # x2
        assert cell[3] == pytest.approx(100.0)  # y2

    def test_find_all_cells_bboxes_without_page_empty_edges(self) -> None:
        """Test that empty explicit edges produce no cells."""
        from tablers import TfSettings, find_all_cells_bboxes

        settings = TfSettings(
            horizontal_strategy="explicit",
            vertical_strategy="explicit",
            explicit_h_edges=[],
            explicit_v_edges=[],
        )

        cells = find_all_cells_bboxes(None, tf_settings=settings)

        assert len(cells) == 0


class TestGetIntersectionsFromEdges:
    """Tests for get_intersections_from_edges function."""

    def test_single_crossing(self) -> None:
        """One h-edge crossing one v-edge produces exactly one intersection."""
        from tablers import get_intersections_from_edges

        h = [Edge("h", 0.0, 50.0, 100.0, 50.0)]
        v = [Edge("v", 50.0, 0.0, 50.0, 100.0)]

        result = get_intersections_from_edges(h, v)

        assert len(result) == 1
        assert (50.0, 50.0) in result

    def test_grid_2x2_produces_nine_intersections(self) -> None:
        """3 h-edges × 3 v-edges form a 2×2 grid with 9 intersection points."""
        from tablers import get_intersections_from_edges

        h = [
            Edge("h", 0.0, 0.0, 100.0, 0.0),
            Edge("h", 0.0, 50.0, 100.0, 50.0),
            Edge("h", 0.0, 100.0, 100.0, 100.0),
        ]
        v = [
            Edge("v", 0.0, 0.0, 0.0, 100.0),
            Edge("v", 50.0, 0.0, 50.0, 100.0),
            Edge("v", 100.0, 0.0, 100.0, 100.0),
        ]

        result = get_intersections_from_edges(h, v)

        assert len(result) == 9
        # Corner points
        assert (0.0, 0.0) in result
        assert (100.0, 100.0) in result
        assert (50.0, 50.0) in result

    def test_empty_edges_return_empty_dict(self) -> None:
        """No edges → no intersections."""
        from tablers import get_intersections_from_edges

        result = get_intersections_from_edges([], [])

        assert result == {}

    def test_no_crossing_edges(self) -> None:
        """H-edge ends before v-edge starts → no intersection."""
        from tablers import get_intersections_from_edges

        h = [Edge("h", 0.0, 50.0, 40.0, 50.0)]  # x range [0, 40]
        v = [Edge("v", 60.0, 0.0, 60.0, 100.0)]  # x = 60

        result = get_intersections_from_edges(h, v)

        assert result == {}

    def test_only_h_edges_no_intersection(self) -> None:
        """Only horizontal edges with no verticals → empty result."""
        from tablers import get_intersections_from_edges

        h = [
            Edge("h", 0.0, 0.0, 100.0, 0.0),
            Edge("h", 0.0, 50.0, 100.0, 50.0),
        ]

        result = get_intersections_from_edges(h, [])

        assert result == {}

    def test_result_structure(self) -> None:
        """Each value in the result dict has 'h' and 'v' keys with Edge lists."""
        from tablers import get_intersections_from_edges

        h = [Edge("h", 0.0, 50.0, 100.0, 50.0)]
        v = [Edge("v", 50.0, 0.0, 50.0, 100.0)]

        result = get_intersections_from_edges(h, v)

        for point, crossing in result.items():
            assert isinstance(point, tuple)
            assert len(point) == 2
            assert isinstance(point[0], float)
            assert isinstance(point[1], float)
            assert "h" in crossing
            assert "v" in crossing
            assert isinstance(crossing["h"], list)
            assert isinstance(crossing["v"], list)

    def test_intersection_point_coordinates(self) -> None:
        """Intersection point is (v.x1, h.y1)."""
        from tablers import get_intersections_from_edges

        h = [Edge("h", 10.0, 30.0, 90.0, 30.0)]
        v = [Edge("v", 70.0, 10.0, 70.0, 80.0)]

        result = get_intersections_from_edges(h, v)

        assert len(result) == 1
        assert (70.0, 30.0) in result

    def test_custom_tolerance_allows_near_miss(self) -> None:
        """A large tolerance makes a near-miss count as an intersection."""
        from tablers import get_intersections_from_edges

        # h-edge ends at x=45, v-edge starts at x=50 (gap = 5)
        h = [Edge("h", 0.0, 50.0, 45.0, 50.0)]
        v = [Edge("v", 50.0, 0.0, 50.0, 100.0)]

        # With tight tolerance the gap is not bridged
        strict = get_intersections_from_edges(h, v, intersection_x_tolerance=1.0)
        assert len(strict) == 0

        # With loose tolerance the gap is bridged
        loose = get_intersections_from_edges(h, v, intersection_x_tolerance=10.0)
        assert len(loose) == 1

    def test_integration_with_get_edges(self, edge_test_doc: "Document") -> None:
        """get_intersections_from_edges works with the output of get_edges."""
        from tablers import get_edges, get_intersections_from_edges

        page = edge_test_doc.get_page(0)
        edges = get_edges(page)

        result = get_intersections_from_edges(edges["h"], edges["v"])

        assert isinstance(result, dict)
        for point, crossing in result.items():
            assert len(point) == 2
            assert "h" in crossing and "v" in crossing
