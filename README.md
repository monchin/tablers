<p align="center">
  <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Rust">
  <img src="https://img.shields.io/badge/Python-3776AB?style=for-the-badge&logo=python&logoColor=white" alt="Python">
</p>

<h1 align="center">⚡ Tablers</h1>

<p align="center">
  <strong>A blazingly fast PDF table extraction library with python API powered by Rust</strong>
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

---

## Features

- 🚀 **Blazingly Fast** - Core algorithms written in Rust for maximum performance
- 🐍 **Pythonic API** - Easy-to-use Python interface with full type hints
- 📄 **Edge Detection** - Accurate table detection using line and rectangle edge analysis
- 📝 **Text Extraction** - Extract text content from table cells with configurable settings
- 📤 **Multiple Export Formats** - Export tables to CSV, Markdown, and HTML
- 🔐 **Encrypted PDFs** - Support for password-protected PDF documents
- 💾 **Memory Efficient** - Lazy page loading for handling large PDF files
- 🖥️ **Cross-Platform** - Works on Windows, Linux, and macOS

## Why Tablers?

This project draws significant inspiration from the table extraction modules of [pdfplumber](https://github.com/jsvine/pdfplumber) and [PyMuPDF](https://github.com/pymupdf/PyMuPDF). Compared to `pdfplumber` and `PyMuPDF`, `tablers` has the following advantages:

- **High Performance**: Utilizes Rust for high-performance PDF processing
- **Higher Accuracy**: Tablers optimizes some table detection algorithms to address table extraction problems that other libraries have not fully solved, including:
    - Mixed strategies where one is text and the other is lines ([#8](https://github.com/monchin/tablers/issues/8))
    - Tables whose edges are actually narrow closepath polylines ([#13](https://github.com/monchin/tablers/issues/13))
    - Extracting table content when the bottom border is absent ([pdfplumber discussion #631](https://github.com/jsvine/pdfplumber/discussions/631))
    - Table recognition when outer lines are missing ([pdfplumber issue #1296](https://github.com/jsvine/pdfplumber/issues/1296))
    - Excluding tables formed by invisible edges ([pdfplumber issue #1357](https://github.com/jsvine/pdfplumber/issues/1357))
- **More Configurable**: Supports customizable table filter settings (`min_rows`, `min_columns`, `include_single_cell`, e.g., see [this issue](https://github.com/pymupdf/PyMuPDF/issues/3987))
- **Clean Python Dependencies**: No external python dependencies required

## Benchmark

Benchmarked on the [ICDAR 2013 Table Competition](https://www.tamirhassan.com/html/competition.html) dataset, evaluating both extraction speed and accuracy across tablers, PyMuPDF, pdfplumber, and camelot. All libraries use their **default configuration** for table extraction. PyMuPDF excludes tables that have only one row or only one column (see [PyMuPDF#3987](https://github.com/pymupdf/PyMuPDF/issues/3987)), and this behaviour is not configurable; among the compared libraries, only **tablers** allows configuring minimum row/column counts. For a fair comparison, the benchmark therefore includes both **tablers (default)** and **tablers (min 2×2)** — the latter with `min_rows=2` and `min_columns=2` so that single-row/single-column tables are filtered out in the same way as in PyMuPDF. For more on the libraries and settings, see the [Libraries compared](https://github.com/monchin/tablers-benchmark#libraries-compared) section in [tablers-benchmark](https://github.com/monchin/tablers-benchmark).

<p align="center">
  <img src="https://raw.githubusercontent.com/monchin/tablers-benchmark/master/table_extraction_benchmark.png" alt="Table Extraction Benchmark">
</p>

For more details, please refer to the [tablers-benchmark](https://github.com/monchin/tablers-benchmark) repository.

## Note

- This solution is primarily designed for text-based PDFs and does not support scanned PDFs.
- **Thread Safety**: `tablers` is **not thread-safe**. The library creates a global PDFium runtime at import time, which is bound to the importing thread. All `Document` operations must be performed on the same thread that imported `tablers`. Using `Document` from a different thread will raise a `PanicException`. For multi-threaded environments, import and use `tablers` within the same worker thread. Use `multiprocessing` for parallel processing instead. See [Thread Safety](docs/usage/advanced.md#thread-safety) for details and code examples.

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

For more advanced usage, please refer to the [documents](https://monchin.github.io/tablers/).

## Requirements

- Python >= 3.10
- Supported platforms: Windows (x64), Linux (x64, ARM64) with glibc >= 2.28, macOS (ARM64)

## License

This project is licensed under the MIT License - see the [LICENSE](https://github.com/monchin/tablers/blob/master/LICENSE) file for details.

## Acknowledgments

- [pdfplumber](https://github.com/jsvine/pdfplumber) - PDF parsing library
- [PyMuPDF](https://github.com/pymupdf/PyMuPDF) - PDF parsing library
- [pdfium-render](https://github.com/ajrcarey/pdfium-render) - Rust bindings for PDFium
- [PyO3](https://github.com/PyO3/pyo3) - Rust bindings for Python
