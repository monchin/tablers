# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.7.1] - 2026-03-20

### Fixed

- **Word extraction panic:** When Pdfium returns no Unicode string for a glyph (`Char.unicode_char` is `None`), word grouping and merging no longer panic. Missing glyphs are skipped like blank characters when `keep_blank_chars` is false, or contribute U+FFFD when merged into word text. Punctuation splitting no longer calls `unwrap()` on an empty string. Regression fixture `tests/data/#28-char-with-no-unicode-info.pdf` is a table-free page used only to ensure these paths do not crash.

## [0.7.0] - 2026-03-03

### Added

- Add `close_unclosed_boundaries` parameter to `TfSettings` (default: `True`). When enabled, a pre-processing pass groups all mutually-intersecting h-edges and v-edges into connected components. For each component, if the h-edges' x-span extends beyond the x-positions of the v-edges in that component (or vice versa for the y direction), a virtual closing edge is synthesised at the extension endpoint. The full intersection-detection and cell-detection pipeline is then re-run with the enhanced edge set. `intersection_x_tolerance` / `intersection_y_tolerance` are used as thresholds. The feature is skipped entirely when either strategy is `"text"` to avoid false positives from text-derived edges. This fixes some known bugs in other table extraction libraries ([pdfplumber#631](https://github.com/jsvine/pdfplumber/discussions/631), [pdfplumber#1296](https://github.com/jsvine/pdfplumber/issues/1296)). (#24)

### Changed

- **Breaking:** `close_unclosed_boundaries` is now `True` by default, which may change table detection results for PDFs whose table outer boundaries are only partially drawn. To restore the previous behavior, set `close_unclosed_boundaries=False` in `TfSettings`. (#24)

## [0.6.0] - 2026-03-01

### Added

- Add `exclude_background_colored_edges` parameter to `TfSettings` (default: `True`) to replace `exclude_white_edges` (#20). Uses a spatial visibility algorithm: for each edge, the fill colors of the rectangles directly adjacent on both sides (within `snap_tolerance`) are inspected. An edge is excluded only when it is indistinguishable from its surroundings on all effective sides (missing sides default to the standard white PDF page background). This correctly handles pages with mixed-background tables and non-white backgrounds. (See tests/data/diff-bg-color-and-line-color.pdf) (#22)

### Fixed

- **Cell text spacing:** A space is now inserted between two words in a table cell only when the gap between their bounding boxes (in reading direction) exceeds the word-extraction tolerance (`x_tolerance` for horizontal text, `y_tolerance` for vertical). This preserves visible gaps in languages like English (e.g. "Table 1" and "Abcd") while avoiding unwanted spaces in languages that do not use spaces between words (e.g. Chinese). (#23)

### Changed

- **Breaking:** `exclude_white_edges` has been removed and replaced by `exclude_background_colored_edges`. The new parameter is `True` by default and subsumes the old behavior on white-background pages. Rename any existing `exclude_white_edges=False` to `exclude_background_colored_edges=False`. (#22)

## [0.5.0] - 2026-02-25 [YANKED]

> **Reason for yanking:** `exclude_white_edges` was not designed comprehensively enough — it failed to handle non-white and mixed-background pages correctly. It has been superseded by the more robust `exclude_background_colored_edges` introduced in 0.6.0.

### Added

- Add `is_stroked` attribute to `Rect` and `Line`: indicates whether the PDF path is stroked (from `path.is_stroked()`)
- Add `fill_mode` attribute to `Rect` and `Line`: fill rule from pdfium-render `PdfPathFillMode` (NONE, WINDING, or EVEN_ODD); expose `FillMode` enum to Python with same three values
- Replace `Line.color` with `Line.stroke_color` and `Line.fill_color` (aligned with `Rect`)
- Add `Table.to_list()` returning `list[list[TableCellValue]]`: each cell has `text` (or `None` when merged), `merged_left`, and `merged_top` so merge direction (left vs above) is explicit (#19)
- Add `TableCellValue` class with attributes `text`, `merged_left`, and `merged_top` for use with `to_list()` (#19)
- Add `get_intersections_from_edges(h_edges, v_edges, ...)` function: given horizontal and vertical edges (as returned by `get_edges`), returns a mapping from every `(x, y)` intersection point to the edges that pass through it; accepts the same tolerance kwargs as `get_edges`
- Add `Document.save_to_bytes()` method to serialize the PDF to an in-memory byte buffer, always without encryption; if the original was password-protected the returned bytes can be opened without a password
- Add `page.doc` back-reference: every `Page` object now carries a reference to the `Document` it belongs to
- Add `Page.page_idx` property: zero-based index of the page within its document
- Add `Page.rotation_degrees` property: clockwise rotation of the page in degrees
- Add `Page.clear_cache()` method as the canonical name for clearing cached objects
- Add `tablers.debug` module with `PageImage` class for visualizing detected tables and edges on a rendered page image; requires the optional `debug` extra (`pip install tablers[debug]`) (#18)
- Add `exclude_white_edges` parameter to `TfSettings` to control filtering of white edges (RGB = 255, 255, 255) during table extraction (#20)

### Changed

- **Breaking:** White edges (RGB = 255, 255, 255) are now excluded by default during table extraction. This may change the behavior of existing code that relied on white edges being included. To restore the previous behavior, set `exclude_white_edges=False` in `TfSettings`. (#20)
- **Breaking:** `Line.color` has been removed. Use `Line.stroke_color` and `Line.fill_color` instead (aligned with `Rect`).
- `Page` is now a Python-level wrapper that holds a `doc` back-reference; Rust-side type is `Pyo3Page`

### Deprecated

- `find_tables_from_cells` parameter `pdf_page` has been renamed to `page` for arguments naming consistency; passing `pdf_page` still works but raises a `DeprecationWarning` and will be removed in a future release
- `Page.clear()` is now an alias for `Page.clear_cache()`; prefer `clear_cache()` going forward

## [0.4.2] - 2026-02-11

### Fixed

- Fix narrow closepath polylines not being regarded as strict lines (#13, #15)
- Fix nested XObject transformation matrices not being applied correctly
- Now python context manager with `Document` would return correct type for better type hinting experience

## [0.4.1] - 2026-02-05

### Changed

- Make this package usable in Linux with glibc >= 2.28 (glibc >= 2.34 formerly)

## [0.4.0] - 2026-01-31

### Added

- Add `clip` parameter to `find_tables` and `find_all_cells_bboxes` for table detection in specific regions (#10)

### Fixed

- Fix edge extension for mixed text/non-text strategies to extract tables correctly (#9)

## [0.3.0] - 2025-01-13

### Added

- Add python `Edge` constructor for programmatic edge creation with `orientation`, `x1`, `y1`, `x2`, `y2`, `width`, and `color` parameters
- Add `explicit` strategy for table detection, allowing the use of explicitly provided edges (#7)
- Add `explicit_h_edges` and `explicit_v_edges` settings to `TfSettings` for providing explicit edges
- Allow `page` parameter to be `None` in `find_tables`, `find_all_cells_bboxes` and `get_edges` when both strategies are `explicit` (and `extract_text` is `False` for `find_tables`)
- Add `plumber_edge_to_tablers_edge` function for converting `pdfplumber` edges to `tablers` edges
- Add documentation and doc workflow with Material-for-MkDocs (#6)

### Changed

- Change `Edge` invalid orientation error from Rust panic to Python `ValueError`
- Change `get_edges` function signature and API

## [0.2.0] - 2025-01-05

### Added

- Add CSV export for tables (`to_csv`) (#5)
- Add Markdown export for tables (`to_markdown`)
- Add HTML export for tables (`to_html`)
- Add `min_rows` and `min_columns` settings for table filtering (default: None, no filter)
- Add `include_single_cell` setting to configure whether to include tables with only one cell (default: false)
- Add `need_strip` option to table extraction functions for whitespace and line feed handling (default: true)
- Add `rows` and `columns` properties for Python bindings

### Fixed

- Fix handling of multiple MoveTo commands in one path segment
- Improve rectangle detection with better path segment type handling

## [0.1.1] - 2025-12-30

### Fixed

- Fix the bug that linux whl does not contains `libpdfium.so` (fixed by renaming it to `libpdfium.so.1`)

## [0.1.0] - 2025-12-30

### Added

- Add NonNegative validations for settings
- Add context manager support to Document class for Python
- Add table finding and text extraction settings with new API functions
- Add comprehensive README with features and usage examples
- Add comprehensive docstrings to Python modules and Rust code
- Add tests
- Add CI workflow
- Add pre-commit hooks

### Changed

- Update TfSettings default strategies from Lines to LinesStrict
- Replace `horizontal_ltr` and `vertical_ttb` with `text_read_in_clockwise` to handle text with rotation_degrees 90 and 270 simultaneously
- Enable to deal with pdf with page_count > 65535 by updating pdfium-render
- Use global pdfium runtime

### Fixed

- Fix cargo clippy errors and update lint scripts
- Replace macOS pdfium dylib with arm64 version

## [0.0.0] - 2025-12-25

### Added

- lines / lines_strict / text strategies for extracting tables in a pdf page
