"""
Tests for edge conversion functions.
"""

import pytest
from tablers import Edge
from tablers.edges import plumber_edge_to_tablers_edge


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
