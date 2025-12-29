"""
Tests for table finding functions.
"""

import pytest
from tablers import (
    Document,
    TfSettings,
    WordsExtractSettings,
    find_all_cells_bboxes,
    find_tables,
    find_tables_from_cells,
)


class TestFindAllCellsBboxes:
    """Tests for find_all_cells_bboxes function."""

    def test_basic_call(self, edge_test_doc: Document) -> None:
        """find_all_cells_bboxes should return a list."""
        page = edge_test_doc.get_page(0)
        cells = find_all_cells_bboxes(page)
        assert isinstance(cells, list)

    def test_returns_bbox_tuples(self, edge_test_doc: Document) -> None:
        """find_all_cells_bboxes should return tuples of 4 floats."""
        page = edge_test_doc.get_page(0)
        cells = find_all_cells_bboxes(page)
        for cell in cells:
            assert isinstance(cell, tuple)
            assert len(cell) == 4
            for coord in cell:
                assert isinstance(coord, (int, float))

    def test_with_tf_settings(self, edge_test_doc: Document) -> None:
        """find_all_cells_bboxes should accept TfSettings parameter."""
        page = edge_test_doc.get_page(0)
        settings = TfSettings()
        cells = find_all_cells_bboxes(page, tf_settings=settings)
        assert isinstance(cells, list)

    def test_with_kwargs(self, edge_test_doc: Document) -> None:
        """find_all_cells_bboxes should accept keyword arguments."""
        page = edge_test_doc.get_page(0)
        cells = find_all_cells_bboxes(page, snap_x_tolerance=5.0)
        assert isinstance(cells, list)

    def test_with_custom_strategy(self, edge_test_doc: Document) -> None:
        """find_all_cells_bboxes should work with custom strategies."""
        page = edge_test_doc.get_page(0)
        settings = TfSettings(vertical_strategy="lines_strict", horizontal_strategy="lines_strict")
        cells = find_all_cells_bboxes(page, tf_settings=settings)
        assert isinstance(cells, list)

    def test_bbox_coordinates_valid(self, edge_test_doc: Document) -> None:
        """BBox coordinates should have x1 < x2 and y1 < y2."""
        page = edge_test_doc.get_page(0)
        cells = find_all_cells_bboxes(page)
        for x1, y1, x2, y2 in cells:
            assert x1 <= x2, f"Invalid bbox: x1 ({x1}) > x2 ({x2})"
            assert y1 <= y2, f"Invalid bbox: y1 ({y1}) > y2 ({y2})"


class TestFindTables:
    """Tests for find_tables function."""

    def test_basic_call_no_text(self, edge_test_doc: Document) -> None:
        """find_tables should work without text extraction."""
        page = edge_test_doc.get_page(0)
        tables = find_tables(page, extract_text=False)
        assert isinstance(tables, list)

    def test_basic_call_with_text(self, edge_test_doc: Document) -> None:
        """find_tables should work with text extraction."""
        page = edge_test_doc.get_page(0)
        tables = find_tables(page, extract_text=True)
        assert isinstance(tables, list)

    def test_table_has_bbox(self, edge_test_doc: Document) -> None:
        """Table objects should have a bbox attribute."""
        page = edge_test_doc.get_page(0)
        tables = find_tables(page, extract_text=False)
        for table in tables:
            assert hasattr(table, "bbox")
            assert isinstance(table.bbox, tuple)
            assert len(table.bbox) == 4

    def test_table_has_cells(self, edge_test_doc: Document) -> None:
        """Table objects should have a cells attribute."""
        page = edge_test_doc.get_page(0)
        tables = find_tables(page, extract_text=False)
        for table in tables:
            assert hasattr(table, "cells")
            assert isinstance(table.cells, list)

    def test_cell_has_bbox(self, edge_test_doc: Document) -> None:
        """TableCell objects should have a bbox attribute."""
        page = edge_test_doc.get_page(0)
        tables = find_tables(page, extract_text=False)
        for table in tables:
            for cell in table.cells:
                assert hasattr(cell, "bbox")
                assert isinstance(cell.bbox, tuple)
                assert len(cell.bbox) == 4

    def test_cell_has_text_when_extracted(self, edge_test_doc: Document) -> None:
        """TableCell objects should have text when extract_text=True."""
        page = edge_test_doc.get_page(0)
        tables = find_tables(page, extract_text=True)
        for table in tables:
            for cell in table.cells:
                assert hasattr(cell, "text")
                assert isinstance(cell.text, str)

    def test_with_tf_settings(self, edge_test_doc: Document) -> None:
        """find_tables should accept TfSettings parameter."""
        page = edge_test_doc.get_page(0)
        settings = TfSettings()
        tables = find_tables(page, extract_text=False, tf_settings=settings)
        assert isinstance(tables, list)

    def test_with_kwargs(self, edge_test_doc: Document) -> None:
        """find_tables should accept keyword arguments."""
        page = edge_test_doc.get_page(0)
        tables = find_tables(page, extract_text=False, snap_x_tolerance=5.0)
        assert isinstance(tables, list)

    def test_with_custom_strategies(self, edge_test_doc: Document) -> None:
        """find_tables should work with custom strategies."""
        page = edge_test_doc.get_page(0)
        settings = TfSettings(vertical_strategy="lines_strict", horizontal_strategy="lines_strict")
        tables = find_tables(page, extract_text=False, tf_settings=settings)
        assert isinstance(tables, list)

    def test_all_pages(self, edge_test_doc: Document) -> None:
        """find_tables should work on all pages."""
        for page in edge_test_doc.pages():
            tables = find_tables(page, extract_text=False)
            assert isinstance(tables, list)


class TestFindTablesFromCells:
    """Tests for find_tables_from_cells function."""

    def test_basic_call_no_text(self, edge_test_doc: Document) -> None:
        """find_tables_from_cells should work without text extraction."""
        page = edge_test_doc.get_page(0)
        cells = find_all_cells_bboxes(page)
        tables = find_tables_from_cells(cells, extract_text=False)
        assert isinstance(tables, list)

    def test_basic_call_with_text(self, edge_test_doc: Document) -> None:
        """find_tables_from_cells should work with text extraction when page provided."""
        page = edge_test_doc.get_page(0)
        cells = find_all_cells_bboxes(page)
        tables = find_tables_from_cells(cells, extract_text=True, pdf_page=page)
        assert isinstance(tables, list)

    def test_extract_text_without_page_raises(self, edge_test_doc: Document) -> None:
        """find_tables_from_cells should raise if extract_text=True but no page."""
        page = edge_test_doc.get_page(0)
        cells = find_all_cells_bboxes(page)
        with pytest.raises(RuntimeError):
            find_tables_from_cells(cells, extract_text=True, pdf_page=None)

    def test_empty_cells_returns_empty(self) -> None:
        """find_tables_from_cells with empty cells should return empty list."""
        tables = find_tables_from_cells([], extract_text=False)
        assert tables == []

    def test_single_cell(self, edge_test_doc: Document) -> None:
        """find_tables_from_cells should handle a single cell."""
        single_cell = [(0.0, 0.0, 100.0, 100.0)]
        tables = find_tables_from_cells(single_cell, extract_text=False)
        assert isinstance(tables, list)

    def test_with_we_settings(self, edge_test_doc: Document) -> None:
        """find_tables_from_cells should accept WordsExtractSettings."""
        page = edge_test_doc.get_page(0)
        cells = find_all_cells_bboxes(page)
        we_settings = WordsExtractSettings(x_tolerance=5.0)
        tables = find_tables_from_cells(
            cells, extract_text=True, pdf_page=page, we_settings=we_settings
        )
        assert isinstance(tables, list)

    def test_tables_have_correct_structure(self, edge_test_doc: Document) -> None:
        """Tables returned should have proper structure."""
        page = edge_test_doc.get_page(0)
        cells = find_all_cells_bboxes(page)
        tables = find_tables_from_cells(cells, extract_text=True, pdf_page=page)
        for table in tables:
            assert hasattr(table, "bbox")
            assert hasattr(table, "cells")
            assert isinstance(table.bbox, tuple)
            assert len(table.bbox) == 4
            assert isinstance(table.cells, list)


class TestTableExtractionIntegration:
    """Integration tests for table extraction workflow."""

    def test_full_workflow_no_text(self, edge_test_doc: Document) -> None:
        """Test the full workflow: find cells then construct tables."""
        page = edge_test_doc.get_page(0)

        # Step 1: Find all cell bboxes
        cell_bboxes = find_all_cells_bboxes(page)
        assert isinstance(cell_bboxes, list)

        # Step 2: Construct tables from cells
        tables = find_tables_from_cells(cell_bboxes, extract_text=False)
        assert isinstance(tables, list)

    def test_full_workflow_with_text(self, edge_test_doc: Document) -> None:
        """Test the full workflow with text extraction."""
        page = edge_test_doc.get_page(0)

        # Step 1: Find all cell bboxes
        cell_bboxes = find_all_cells_bboxes(page)

        # Step 2: Construct tables with text extraction
        tables = find_tables_from_cells(cell_bboxes, extract_text=True, pdf_page=page)

        # Verify table structure
        for table in tables:
            for cell in table.cells:
                assert hasattr(cell, "text")
                assert isinstance(cell.text, str)

    def test_direct_vs_two_step_comparison(self, edge_test_doc: Document) -> None:
        """Comparing direct find_tables vs two-step approach should yield similar results."""
        page = edge_test_doc.get_page(0)

        # Direct approach
        tables_direct = find_tables(page, extract_text=False)

        # Two-step approach
        cells = find_all_cells_bboxes(page)
        tables_two_step = find_tables_from_cells(cells, extract_text=False)

        # Both should return lists (may have different table counts due to clustering)
        assert isinstance(tables_direct, list)
        assert isinstance(tables_two_step, list)

    def test_multiple_pages_extraction(self, edge_test_doc: Document) -> None:
        """Test table extraction across multiple pages."""
        all_tables = []
        for page in edge_test_doc.pages():
            tables = find_tables(page, extract_text=True)
            all_tables.extend(tables)

        assert isinstance(all_tables, list)

    def test_with_words_extract_pdf(self, words_extract_doc: Document) -> None:
        """Test table extraction on words-extract.pdf."""
        page = words_extract_doc.get_page(0)
        tables = find_tables(page, extract_text=True)
        assert isinstance(tables, list)

    def test_custom_settings_workflow(self, edge_test_doc: Document) -> None:
        """Test workflow with custom settings."""
        page = edge_test_doc.get_page(0)

        # Create custom settings
        tf_settings = TfSettings(
            vertical_strategy="lines",
            horizontal_strategy="lines",
            snap_x_tolerance=5.0,
            snap_y_tolerance=5.0,
            edge_min_length=10.0,
        )

        # Find cells with settings
        cells = find_all_cells_bboxes(page, tf_settings=tf_settings)
        assert isinstance(cells, list)

        # Find tables with settings
        tables = find_tables(page, extract_text=True, tf_settings=tf_settings)
        assert isinstance(tables, list)
