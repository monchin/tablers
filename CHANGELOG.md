# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Add `Table.to_list()` returning `list[list[TableCellValue]]`: each cell has `text` (or `None` when merged), `merged_left`, and `merged_top` so merge direction (left vs above) is explicit
- Add `TableCellValue` class with attributes `text`, `merged_left`, and `merged_top` for use with `to_list()`
- Add `get_intersections_from_edges(h_edges, v_edges, ...)` function: given horizontal and vertical edges (as returned by `get_edges`), returns a mapping from every `(x, y)` intersection point to the edges that pass through it; accepts the same tolerance kwargs as `get_edges`
- Add `Document.save_to_bytes()` method to serialize the PDF to an in-memory byte buffer, always without encryption; if the original was password-protected the returned bytes can be opened without a password
- Add `page.doc` back-reference: every `Page` object now carries a reference to the `Document` it belongs to
- Add `Page.page_idx` property: zero-based index of the page within its document
- Add `Page.rotation_degrees` property: clockwise rotation of the page in degrees
- Add `Page.clear_cache()` method as the canonical name for clearing cached objects
- Add `tablers.debug` module with `PageImage` class for visualizing detected tables and edges on a rendered page image; requires the optional `debug` extra (`pip install tablers[debug]`)

### Changed

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
