"""Tests for pickle support of PyO3 data objects."""

import pickle

import pytest
from tablers import (
    Edge,
    TfSettings,
    WordsExtractSettings,
    find_tables,
)
from tablers.tablers import FillMode


class TestEdgePickle:
    def test_pickle_round_trip(self):
        edge = Edge("h", 10.0, 20.0, 30.0, 40.0, width=2.0, color=(255, 0, 0, 128))
        restored = pickle.loads(pickle.dumps(edge))
        assert restored.orientation == "h"
        assert restored.x1 == 10.0
        assert restored.y1 == 20.0
        assert restored.x2 == 30.0
        assert restored.y2 == 40.0
        assert restored.width == 2.0
        assert restored.color == (255, 0, 0, 128)

    def test_pickle_vertical_edge(self):
        edge = Edge("v", 5.0, 10.0, 5.0, 50.0)
        restored = pickle.loads(pickle.dumps(edge))
        assert restored.orientation == "v"
        assert restored.x1 == 5.0
        assert restored.y1 == 10.0

    def test_pickle_default_edge(self):
        edge = Edge("h", 0.0, 0.0, 100.0, 0.0)
        restored = pickle.loads(pickle.dumps(edge))
        assert restored.width == 1.0
        assert restored.color == (0, 0, 0, 255)


class TestTableCellPickle:
    def test_pickle_from_extracted_table(self, edge_test_doc):
        tables = find_tables(edge_test_doc.get_page(0), extract_text=True)
        if not tables:
            pytest.skip("No tables found")
        cell = tables[0].cells[0]
        restored = pickle.loads(pickle.dumps(cell))
        assert restored.text == cell.text
        assert restored.bbox == cell.bbox


class TestTablePickle:
    def test_pickle_table_no_text(self, edge_test_doc):
        tables = find_tables(edge_test_doc.get_page(0), extract_text=False)
        if not tables:
            pytest.skip("No tables found")
        table = tables[0]
        restored = pickle.loads(pickle.dumps(table))
        assert restored.bbox == table.bbox
        assert len(restored.cells) == len(table.cells)
        assert restored.page_index == table.page_index
        assert restored.text_extracted == table.text_extracted

    def test_pickle_table_with_text(self, multiple_move_to_in_one_seg_doc):
        tables = find_tables(multiple_move_to_in_one_seg_doc.get_page(0), extract_text=True)
        assert len(tables) == 1
        table = tables[0]
        restored = pickle.loads(pickle.dumps(table))
        assert restored.bbox == table.bbox
        assert len(restored.cells) == len(table.cells)
        assert restored.text_extracted is True
        for orig, rest in zip(table.cells, restored.cells, strict=True):
            assert orig.text == rest.text
            assert orig.bbox == rest.bbox

    def test_pickle_table_then_to_csv(self, multiple_move_to_in_one_seg_doc):
        """A pickled+unpickled table should still produce correct to_csv()."""
        tables = find_tables(multiple_move_to_in_one_seg_doc.get_page(0), extract_text=True)
        table = tables[0]
        restored = pickle.loads(pickle.dumps(table))
        assert restored.to_csv() == table.to_csv()

    def test_pickle_list_of_tables(self, edge_test_doc):
        """A list of tables should be picklable."""
        tables = find_tables(edge_test_doc.get_page(0), extract_text=True)
        restored_list = pickle.loads(pickle.dumps(tables))
        assert len(restored_list) == len(tables)
        for orig, rest in zip(tables, restored_list, strict=True):
            assert orig.bbox == rest.bbox
            assert len(orig.cells) == len(rest.cells)


class TestPyTableCellValuePickle:
    def test_pickle_from_to_list(self, multiple_move_to_in_one_seg_doc):
        tables = find_tables(multiple_move_to_in_one_seg_doc.get_page(0), extract_text=True)
        rows = tables[0].to_list()
        for row in rows:
            for cell in row:
                restored = pickle.loads(pickle.dumps(cell))
                assert restored.text == cell.text
                assert restored.merged_left == cell.merged_left
                assert restored.merged_top == cell.merged_top


class TestPyCellGroupPickle:
    def test_pickle_rows(self, edge_test_doc):
        tables = find_tables(edge_test_doc.get_page(0), extract_text=True)
        if not tables:
            pytest.skip("No tables found")
        table = tables[0]
        rows = table.rows
        for row in rows:
            restored = pickle.loads(pickle.dumps(row))
            assert restored.bbox == row.bbox
            assert len(restored.cells) == len(row.cells)


class TestCharPickle:
    def test_pickle_from_objects(self, edge_test_doc):
        page = edge_test_doc.get_page(0)
        page.extract_objects()
        objects = page.objects
        if not objects or not objects.chars:
            pytest.skip("Need chars")
        char = objects.chars[0]
        restored = pickle.loads(pickle.dumps(char))
        assert restored.unicode_char == char.unicode_char
        assert restored.bbox == char.bbox
        assert restored.rotation_degrees == char.rotation_degrees
        assert restored.upright == char.upright


class TestRectPickle:
    def test_pickle_from_objects(self, edge_test_doc):
        page = edge_test_doc.get_page(0)
        page.extract_objects()
        objects = page.objects
        if not objects or not objects.rects:
            pytest.skip("Need rects")
        rect = objects.rects[0]
        restored = pickle.loads(pickle.dumps(rect))
        assert restored.bbox == rect.bbox
        assert restored.fill_color == rect.fill_color
        assert restored.stroke_color == rect.stroke_color
        assert restored.stroke_width == rect.stroke_width
        assert restored.is_stroked == rect.is_stroked
        assert restored.fill_mode == rect.fill_mode


class TestLinePickle:
    def test_pickle_from_objects(self, edge_test_doc):
        page = edge_test_doc.get_page(0)
        page.extract_objects()
        objects = page.objects
        if not objects or not objects.lines:
            pytest.skip("Need lines")
        line = objects.lines[0]
        restored = pickle.loads(pickle.dumps(line))
        assert restored.line_type == line.line_type
        assert restored.points == line.points
        assert restored.stroke_color == line.stroke_color
        assert restored.fill_color == line.fill_color
        assert restored.width == line.width
        assert restored.is_stroked == line.is_stroked
        assert restored.fill_mode == line.fill_mode


class TestObjectsPickle:
    def test_pickle_objects(self, edge_test_doc):
        page = edge_test_doc.get_page(0)
        page.extract_objects()
        objects = page.objects
        if not objects:
            pytest.skip("Need objects")
        restored = pickle.loads(pickle.dumps(objects))
        assert len(restored.rects) == len(objects.rects)
        assert len(restored.lines) == len(objects.lines)
        assert len(restored.chars) == len(objects.chars)


class TestFillModePickle:
    def test_pickle_fill_mode_values(self):
        for val in [FillMode.NONE, FillMode.WINDING, FillMode.EVEN_ODD]:
            restored = pickle.loads(pickle.dumps(val))
            assert restored == val


class TestWordsExtractSettingsPickle:
    def test_pickle_default(self):
        s = WordsExtractSettings()
        restored = pickle.loads(pickle.dumps(s))
        assert restored.x_tolerance == s.x_tolerance
        assert restored.y_tolerance == s.y_tolerance
        assert restored.keep_blank_chars == s.keep_blank_chars
        assert restored.use_text_flow == s.use_text_flow
        assert restored.text_read_in_clockwise == s.text_read_in_clockwise
        assert restored.split_at_punctuation == s.split_at_punctuation
        assert restored.expand_ligatures == s.expand_ligatures
        assert restored.need_strip == s.need_strip

    def test_pickle_custom(self):
        s = WordsExtractSettings(
            x_tolerance=5.0,
            y_tolerance=10.0,
            keep_blank_chars=True,
            split_at_punctuation="all",
            need_strip=False,
        )
        restored = pickle.loads(pickle.dumps(s))
        assert restored.x_tolerance == 5.0
        assert restored.y_tolerance == 10.0
        assert restored.keep_blank_chars is True
        assert restored.split_at_punctuation == "all"
        assert restored.need_strip is False


class TestTfSettingsPickle:
    def test_pickle_default(self):
        s = TfSettings()
        restored = pickle.loads(pickle.dumps(s))
        assert restored.vertical_strategy == s.vertical_strategy
        assert restored.horizontal_strategy == s.horizontal_strategy
        assert restored.snap_x_tolerance == s.snap_x_tolerance
        assert restored.include_single_cell == s.include_single_cell

    def test_pickle_custom(self):
        s = TfSettings(
            vertical_strategy="lines",
            horizontal_strategy="text",
            snap_x_tolerance=5.0,
            min_rows=2,
            min_columns=3,
            text_split_at_punctuation=".,;",
        )
        restored = pickle.loads(pickle.dumps(s))
        assert restored.vertical_strategy == "lines"
        assert restored.horizontal_strategy == "text"
        assert restored.snap_x_tolerance == 5.0
        assert restored.min_rows == 2
        assert restored.min_columns == 3
        assert restored.text_split_at_punctuation == ".,;"

    def test_pickle_with_explicit_edges(self):
        h_edges = [Edge("h", 0, 0, 100, 0), Edge("h", 0, 50, 100, 50)]
        v_edges = [Edge("v", 0, 0, 0, 50), Edge("v", 100, 0, 100, 50)]
        s = TfSettings(
            horizontal_strategy="explicit",
            vertical_strategy="explicit",
            explicit_h_edges=h_edges,
            explicit_v_edges=v_edges,
        )
        restored = pickle.loads(pickle.dumps(s))
        assert len(restored.explicit_h_edges) == 2
        assert len(restored.explicit_v_edges) == 2


class TestMultiprocessingWithPickle:
    def test_table_picklable_for_multiprocessing(self, edge_test_doc):
        """Tables extracted should be directly picklable for multiprocessing."""
        tables = find_tables(edge_test_doc.get_page(0), extract_text=True)
        if not tables:
            pytest.skip("No tables found")
        for table in tables:
            data = pickle.dumps(table)
            restored = pickle.loads(data)
            assert len(restored.cells) == len(table.cells)
            assert restored.bbox == table.bbox
            assert restored.to_csv() == table.to_csv()

    def test_pickle_table_rows_columns(self, edge_test_doc):
        """Rows and columns should be correct after pickle round-trip."""
        tables = find_tables(edge_test_doc.get_page(0), extract_text=True)
        if not tables:
            pytest.skip("No tables found")
        table = tables[0]
        restored = pickle.loads(pickle.dumps(table))
        assert len(restored.rows) == len(table.rows)
        assert len(restored.columns) == len(table.columns)

    def test_pickle_table_then_to_list(self, edge_test_doc):
        """to_list() should produce identical text content after pickle round-trip."""
        tables = find_tables(edge_test_doc.get_page(0), extract_text=True)
        if not tables:
            pytest.skip("No tables found")
        table = tables[0]
        restored = pickle.loads(pickle.dumps(table))
        if table.text_extracted:
            orig_rows = table.to_list()
            rest_rows = restored.to_list()
            assert len(rest_rows) == len(orig_rows)
            for orig_row, rest_row in zip(orig_rows, rest_rows, strict=True):
                assert len(rest_row) == len(orig_row)
                for orig_cell, rest_cell in zip(orig_row, rest_row, strict=True):
                    assert rest_cell.text == orig_cell.text
                    assert rest_cell.merged_left == orig_cell.merged_left
                    assert rest_cell.merged_top == orig_cell.merged_top


class TestPickleProtocols:
    """Verify pickle works across all standard protocol versions."""

    def test_edge_all_protocols(self):
        edge = Edge("h", 10.0, 20.0, 30.0, 40.0, width=2.0, color=(255, 0, 0, 128))
        for protocol in range(pickle.HIGHEST_PROTOCOL + 1):
            restored = pickle.loads(pickle.dumps(edge, protocol=protocol))
            assert restored.orientation == edge.orientation
            assert restored.x1 == edge.x1

    def test_table_all_protocols(self, edge_test_doc):
        tables = find_tables(edge_test_doc.get_page(0), extract_text=True)
        if not tables:
            pytest.skip("No tables found")
        table = tables[0]
        for protocol in range(pickle.HIGHEST_PROTOCOL + 1):
            restored = pickle.loads(pickle.dumps(table, protocol=protocol))
            assert len(restored.cells) == len(table.cells)
            assert restored.bbox == table.bbox

    def test_tf_settings_all_protocols(self):
        s = TfSettings(vertical_strategy="lines", horizontal_strategy="text", snap_x_tolerance=5.0)
        for protocol in range(pickle.HIGHEST_PROTOCOL + 1):
            restored = pickle.loads(pickle.dumps(s, protocol=protocol))
            assert restored.vertical_strategy == s.vertical_strategy
