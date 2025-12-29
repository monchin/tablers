<p align="center">
  <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Rust">
  <img src="https://img.shields.io/badge/Python-3776AB?style=for-the-badge&logo=python&logoColor=white" alt="Python">
</p>

<h1 align="center">⚡ Tablers</h1>

<p align="center">
  <strong>A blazingly fast PDF table extraction library powered by Rust</strong>
</p>

<p align="center">
  <a href="https://github.com/monchin/tablers/blob/main/LICENSE">
    <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT">
  </a>
  <a href="https://pypi.org/project/tablers/">
    <img src="https://img.shields.io/pypi/v/tablers.svg" alt="PyPI version">
  </a>
  <a href="https://pypi.org/project/tablers/">
    <img src="https://img.shields.io/pypi/pyversions/tablers.svg" alt="Python versions">
  </a>
  <a href="https://pdm-project.org">
    <img src="https://img.shields.io/endpoint?url=https%3A%2F%2Fcdn.jsdelivr.net%2Fgh%2Fpdm-project%2F.github%2Fbadge.json" alt="pdm-managed">
  </a>
</p>

<p align="center">
  <a href="#features">Features</a> •
  <a href="#motivation">Motivation</a> •
  <a href="#note">Note</a> •
  <a href="#installation">Installation</a> •
  <a href="#quick-start">Quick Start</a> •
  <a href="#api-reference">API Reference</a> •
  <a href="#advanced-usage">Advanced Usage</a> •
  <a href="#license">License</a>
</p>

---

## Features

- 🚀 **Blazingly Fast** - Core algorithms written in Rust for maximum performance
- 🐍 **Pythonic API** - Easy-to-use Python interface with full type hints
- 📄 **Edge Detection** - Accurate table detection using line and rectangle edge analysis
- 📝 **Text Extraction** - Extract text content from table cells with configurable settings
- 🔐 **Encrypted PDFs** - Support for password-protected PDF documents
- 💾 **Memory Efficient** - Lazy page loading for handling large PDF files
- 🖥️ **Cross-Platform** - Works on Windows, Linux, and macOS

## Motivation
This project draws significant inspiration from the table extraction modules of [pdfplumber](https://github.com/jsvine/pdfplumber) and [PyMuPDF](https://github.com/pymupdf/PyMuPDF). During practical use, we observed that both pdfplumber and PyMuPDF exhibit performance limitations when extracting tables from large PDF files, which hindered efficient data processing. To address this issue, the current project was initiated.

## Note
This solution is primarily designed for text-based PDFs and does not support scanned PDFs.

## Installation

```bash
pip install tablers
```

## Quick Start

### Basic Table Extraction

```python
from tablers import Document, find_tables

# Open a PDF document
doc = Document("example.pdf")

# Extract tables from each page
for page in doc.pages():
    tables = find_tables(page, extract_text=True)
    for table in tables:
        print(f"Found table with {len(table.cells)} cells")
        for cell in table.cells:
            print(f"  Cell: {cell.text} at {cell.bbox}")

doc.close()
```

### Using Context Manager

```python
from tablers import Document, find_tables

with Document("example.pdf") as doc:
    page = doc.get_page(0)  # Get first page
    tables = find_tables(page, extract_text=True)
    
    for table in tables:
        print(f"Table bbox: {table.bbox}")
```

### Opening PDF from Bytes

```python
from tablers import Document, find_tables

with open("example.pdf", "rb") as f:
    pdf_bytes = f.read()

doc = Document(bytes=pdf_bytes)
# ... process document
doc.close()
```

### Working with Encrypted PDFs

```python
from tablers import Document

doc = Document("encrypted.pdf", password="secret123")
# ... process document
doc.close()
```

## API Reference

### Document

The main class for working with PDF documents.

```python
class Document:
    def __init__(
        self,
        path: Path | str | None = None,
        bytes: bytes | None = None,
        password: str | None = None
    ): ...
    
    @property
    def page_count(self) -> int: ...
    
    def get_page(self, page_num: int) -> Page: ...
    def pages(self) -> PageIterator: ...
    def close(self) -> None: ...
    def is_closed(self) -> bool: ...
```

### find_tables

Extract all tables from a page.

```python
def find_tables(
    page: Page,
    extract_text: bool,
    tf_settings: TfSettings | None = None,
    **kwargs
) -> list[Table]: ...
```

### find_all_cells_bboxes

Find all table cell bounding boxes in a page.

```python
def find_all_cells_bboxes(
    page: Page,
    tf_settings: TfSettings | None = None,
    **kwargs
) -> list[tuple[float, float, float, float]]: ...
```

### find_tables_from_cells

Construct tables from pre-detected cell bounding boxes.

```python
def find_tables_from_cells(
    cells: list[tuple[float, float, float, float]],
    extract_text: bool,
    pdf_page: Page | None = None,
    we_settings: WordsExtractSettings | None = None,
    **kwargs
) -> list[Table]: ...
```

### Table

Represents an extracted table.

```python
class Table:
    @property
    def cells(self) -> list[TableCell]: ...
    
    @property
    def bbox(self) -> tuple[float, float, float, float]: ...
    
    @property
    def page_index(self) -> int: ...
    
    @property
    def text_extracted(self) -> bool: ...
```

### TableCell

Represents a single cell in a table.

```python
class TableCell:
    @property
    def text(self) -> str: ...
    
    @property
    def bbox(self) -> tuple[float, float, float, float]: ...
```

## Advanced Usage

### Custom Table Detection Settings

Fine-tune the table detection algorithm with `TfSettings`:

```python
from tablers import Document, find_tables, TfSettings

settings = TfSettings(
    vertical_strategy="lines",       # "lines", "lines_strict", "text"
    horizontal_strategy="lines",     # "lines", "lines_strict", "text"
    snap_x_tolerance=5.0,            # X-axis snapping tolerance
    snap_y_tolerance=5.0,            # Y-axis snapping tolerance
    edge_min_length=10.0,            # Minimum edge length
)

with Document("complex_table.pdf") as doc:
    page = doc.get_page(0)
    tables = find_tables(page, extract_text=True, tf_settings=settings)
```

### Custom Text Extraction Settings

Configure text extraction with `WordsExtractSettings`:

```python
from tablers import Document, find_tables_from_cells, find_all_cells_bboxes, WordsExtractSettings

we_settings = WordsExtractSettings(
    x_tolerance=3.0,     # Horizontal tolerance for word grouping
    y_tolerance=3.0,     # Vertical tolerance for word grouping
)

with Document("example.pdf") as doc:
    page = doc.get_page(0)
    cells = find_all_cells_bboxes(page)
    tables = find_tables_from_cells(
        cells,
        extract_text=True,
        pdf_page=page,
        we_settings=we_settings
    )
```

### Two-Step Table Extraction

For more control, separate cell detection from table construction:

```python
from tablers import Document, find_all_cells_bboxes, find_tables_from_cells

with Document("example.pdf") as doc:
    page = doc.get_page(0)
    
    # Step 1: Detect all cell bounding boxes
    cell_bboxes = find_all_cells_bboxes(page)
    print(f"Found {len(cell_bboxes)} cells")
    
    # Step 2: Optionally filter or modify cell_bboxes here
    
    # Step 3: Construct tables from cells
    tables = find_tables_from_cells(
        cell_bboxes,
        extract_text=True,
        pdf_page=page
    )
```

### Processing Multiple Pages

```python
from tablers import Document, find_tables

with Document("multi_page.pdf") as doc:
    print(f"Processing {doc.page_count} pages")
    
    all_tables = []
    for page in doc.pages():
        tables = find_tables(page, extract_text=True)
        all_tables.extend(tables)
    
    print(f"Found {len(all_tables)} tables in total")
```

## Development

### Building from Source
#### Prerequisites
```bash
install rustup
install uv

uv tool install maturin pdm pre-commit
uv python install 3.10(or higher)
```
```bash
# Clone the repository
git clone https://github.com/monchin/tablers.git
cd tablers

# Install dependencies
pdm sync

# Build the Rust extension
maturin develop --uv

# Run tests
pdm test
```

## Requirements

- Python >= 3.10
- Supported platforms: Windows (x64), Linux (x64), macOS (ARM64)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [pdfium-render](https://github.com/nickswalker/pdfium-render) - Rust bindings for PDFium
- [PyO3](https://github.com/PyO3/pyo3) - Rust bindings for Python
