# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
