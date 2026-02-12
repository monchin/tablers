# AGENTS.md

## Project Overview

**Tablers** is a blazingly fast PDF table extraction library that combines the performance of Rust with the convenience of Python. The project is designed to extract tables from text-based PDFs using edge detection and cell identification algorithms.

### Key Technologies

- **Language**: Rust (core) + Python (API)
- **Python Bindings**: PyO3
- **PDF Engine**: Pdfium (via pdfium-render)
- **Build System**: Maturin
- **Package Manager**: PDM (Python Development Master)

## Project Structure

```text
tablers/
├── src/                    # Rust source code
│   ├── lib.rs             # Main library entry point, PyO3 bindings
│   ├── edges.rs           # Edge detection algorithms
│   ├── tables.rs          # Table finding and construction logic
│   ├── pages.rs           # PDF page handling
│   ├── objects.rs         # PDF object extraction (chars, lines, rects)
│   ├── words.rs           # Word extraction and text processing
│   ├── clusters.rs        # Clustering algorithms for grouping objects
│   ├── settings.rs        # Configuration and settings structures
│   └── test_utils.rs      # Testing utilities
├── python/tablers/        # Python package
│   ├── __init__.py        # Python API wrapper and runtime initialization
│   ├── tablers.pyi        # Type stubs for Python
│   ├── edges.py           # Additional Python edge utilities
│   └── [pdfium binaries]  # Platform-specific Pdfium libraries
├── tests/                 # Test suite
│   ├── python/            # Python tests
│   └── data/              # Test PDF files
├── docs/                  # Documentation (MkDocs)
├── Cargo.toml             # Rust dependencies and configuration
├── pyproject.toml         # Python project configuration
└── README.md              # Project documentation
```

## Architecture

### Core Components

#### 1. **Document Management** (`lib.rs`)

- `PdfiumRuntime`: Manages the Pdfium library lifecycle (singleton pattern)
- `Document`: Represents an opened PDF document with lazy page loading
- `PyPage`: Represents a single PDF page with caching support

#### 2. **Edge Detection** (`edges.rs`)

- Detects horizontal and vertical edges from:
  - Explicit lines in the PDF
  - Rectangle borders
  - Text boundaries (implicit edges)
- Supports multiple detection strategies:
  - `lines`: Detect from explicit lines
  - `lines_strict`: Strict line detection
  - `text`: Infer edges from text layout
  - `explicit`: Use user-provided edges

#### 3. **Table Finding** (`tables.rs`)

- `TableFinder`: Main algorithm for table detection
- Cell identification through edge intersection
- Table construction from cells with filtering options:
  - `min_rows`, `min_columns`: Minimum table dimensions
  - `include_single_cell`: Include single-cell tables
  - Configurable snap tolerances for alignment

#### 4. **Object Extraction** (`objects.rs`, `pages.rs`)

- Extracts PDF primitives:
  - Characters with bounding boxes
  - Lines (explicit path objects)
  - Rectangles (filled or stroked)
- Caching system for performance optimization

#### 5. **Text Processing** (`words.rs`)

- Groups characters into words based on proximity
- Configurable word extraction settings:
  - `horizontal_tolerance`: Max horizontal gap between chars
  - `vertical_tolerance`: Max vertical gap for same line
  - `keep_blank_chars`: Include whitespace characters

### Python API Layer

The `python/tablers/__init__.py` module provides a Pythonic wrapper:

```python
# High-level Document class with context manager support
class Document:
    def __init__(path=None, bytes=None, password=None)
    def get_page(page_num: int) -> Page
    def pages() -> PageIterator
    def close()

# Main table extraction functions
def find_tables(page, extract_text=True, clip=None, **kwargs) -> List[Table]
def find_all_cells_bboxes(page, clip=None, **kwargs) -> List[Tuple[float, float, float, float]]
def find_tables_from_cells(cells, extract_text, pdf_page=None, **kwargs) -> List[Table]

# Edge extraction
def get_edges(page, **kwargs) -> Dict[str, List[Edge]]
```

## Key Features

### 1. **High Performance**

- Rust implementation for CPU-intensive operations
- Memory-efficient lazy page loading
- Caching system for extracted objects
- Zero-copy where possible using PyO3

### 2. **Flexibility**

- Multiple edge detection strategies
- Configurable filtering options
- Clip region support for partial page extraction
- Custom explicit edge support

### 3. **Cross-Platform**

- Windows (x64)
- Linux (x64, glibc >= 2.28)
- macOS (ARM64)
- Bundled Pdfium binaries for each platform

### 4. **Developer-Friendly**

- Full type hints (`.pyi` stubs)
- Context manager support
- Comprehensive test suite
- Encrypted PDF support

## Development Workflow

### Building the Project

```bash
# Install dependencies
pdm sync

# Build Rust extension
maturin develop [--uv]    # Development build

# Format code
pdm run fmt         # Format Rust and Python code

# Run linters
pdm run lint        # Check Rust and Python code

# Run tests
pdm run test        # Run all tests (Rust + Python)
```

### Testing Strategy

1. **Rust Tests**: Unit tests in `src/` modules using `#[cfg(test)]`
2. **Python Tests**: Integration tests in `tests/python/` using pytest
3. **Test Data**: Sample PDFs in `tests/data/` including:
   - Edge detection tests
   - Encrypted PDFs
   - Various table layouts

### Code Quality Tools

- **Rust**: `cargo fmt`, `cargo clippy`, `cargo check`
- **Python**: `ruff` (linting and formatting), `ty` (type checking)
- **Markdown**: `rumdl` (Markdown linting)

## Important Implementation Details

### Memory Safety

- Uses `Rc<RefCell<>>` for shared mutable state in Rust
- `unsendable` PyO3 classes (Pdfium is not thread-safe)
- Lifetime transmutation for document references (carefully managed)

### Global State Management

- `OnceLock<Pdfium>` for singleton Pdfium instance
- Ensures `bind_to_library` is only called once per process
- Thread-safe initialization with reuse on subsequent calls

### Error Handling

- Rust: `Result<T, PdfiumError>` with `thiserror` for custom errors
- Python: Converted to appropriate Python exceptions via PyO3

### Performance Optimizations

- Object extraction caching per page
- Lazy page loading via iterator pattern
- Efficient bounding box operations with `OrderedFloat`
- Itertools for Rust iterator optimization

## Common Tasks for AI Agents

### Adding a New Feature

1. **Identify the layer**: Rust core or Python wrapper?
2. **Rust changes**:
   - Add logic to appropriate module (`edges.rs`, `tables.rs`, etc.)
   - Add PyO3 bindings in `lib.rs` if needed
   - Write Rust unit tests
3. **Python changes**:
   - Update `__init__.py` if wrapper logic needed
   - Update type stubs in `tablers.pyi`
   - Add to `__all__` export list
4. **Testing**: Add Python integration tests in `tests/python/`
5. **Documentation**: Update docs in `docs/` directory

### Debugging Issues

1. **Check test PDFs**: Use files in `tests/data/`
2. **Enable Rust backtraces**: `RUST_BACKTRACE=1`
3. **Python debugging**: Use pytest with `-v` or `-s` flags
4. **Edge detection**: Use `get_edges()` to inspect detected edges
5. **Object extraction**: Check `page.extract_objects()` and `page.objects`

### Modifying Settings

Settings are defined in `src/settings.rs`:

- `TfSettings`: Table finder settings (strategies, tolerances, filters)
- `WordsExtractSettings`: Word extraction configuration

Both are exposed to Python via PyO3 `#[pyclass]` with `#[pyo3(signature=...)]` for kwargs.

## Dependencies

### Rust Crates

- `pyo3`: Python bindings (with abi3-py310 for stable ABI)
- `pdfium-render`: PDF rendering and parsing
- `itertools`: Iterator utilities
- `ordered-float`: Float comparison for bounding boxes
- `once_cell`: Global state management
- `thiserror`: Error handling

### Python Dependencies

- **Development only** (no runtime dependencies):
  - `pytest`: Testing framework
  - `pytest-cov`: Coverage reporting

## Release Process

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Run full test suite: `pdm run test`
4. Build wheels with Maturin (via CI/CD)
5. Publish to PyPI

## Resources

- **Documentation**: <https://monchin.github.io/tablers/>
- **Repository**: <https://github.com/monchin/tablers>
- **PyPI**: <https://pypi.org/project/tablers/>
- **Benchmark**: <https://github.com/monchin/tablers-benchmark>

## Inspiration and References

This project is inspired by:

- [pdfplumber](https://github.com/jsvine/pdfplumber): Python PDF parsing library
- [PyMuPDF](https://github.com/pymupdf/PyMuPDF): Fast PDF toolkit

Key improvements over inspirations:

- Higher performance via Rust implementation
- More configurable table filtering
- Zero Python runtime dependencies
- Better type safety with full type hints
