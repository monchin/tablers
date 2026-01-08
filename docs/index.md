# Tablers

<p align="center">
  <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Rust">
  <img src="https://img.shields.io/badge/Python-3776AB?style=for-the-badge&logo=python&logoColor=white" alt="Python">
</p>

<p align="center">
  <strong>⚡ A blazingly fast PDF table extraction library powered by Rust</strong>
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
</p>

---

## Features

- :rocket: **Blazingly Fast** - Core algorithms written in Rust for maximum performance
- :snake: **Pythonic API** - Easy-to-use Python interface with full type hints
- :page_facing_up: **Edge Detection** - Accurate table detection using line and rectangle edge analysis
- :memo: **Text Extraction** - Extract text content from table cells with configurable settings
- :outbox_tray: **Multiple Export Formats** - Export tables to CSV, Markdown, and HTML
- :lock: **Encrypted PDFs** - Support for password-protected PDF documents
- :floppy_disk: **Memory Efficient** - Lazy page loading for handling large PDF files
- :computer: **Cross-Platform** - Works on Windows, Linux, and macOS

## Motivation

This project draws significant inspiration from the table extraction modules of [pdfplumber](https://github.com/jsvine/pdfplumber) and [PyMuPDF](https://github.com/pymupdf/PyMuPDF). During practical use, we observed that both pdfplumber and PyMuPDF exhibit performance limitations when extracting tables from large PDF files, which hindered efficient data processing. To address this issue, the current project was initiated.

## Benchmark

Performance comparison of tablers, pymupdf and pdfplumber for PDF table extraction:

<p align="center">
  <img src="https://raw.githubusercontent.com/monchin/tablers-benchmark/master/table_extraction_benchmark.png" alt="Table Extraction Benchmark">
</p>

For more details, please refer to the [tablers-benchmark](https://github.com/monchin/tablers-benchmark) repository.

## Quick Start

### Installation

```bash
pip install tablers
```

### Basic Usage

```python
from tablers import Document, find_tables

# Open a PDF document
with Document("example.pdf") as doc:
    # Iterate through pages
    for page in doc.pages():
        # Extract tables
        tables = find_tables(page, extract_text=True)
        for table in tables:
            print(f"Found table with {len(table.cells)} cells")
            # Export to Markdown
            print(table.to_markdown())
```

## Note

This solution is primarily designed for text-based PDFs and does not support scanned PDFs.

## Requirements

- Python >= 3.10
- Supported platforms: Windows (x64), Linux (x64) with glibc >= 2.34, macOS (ARM64)

## License

This project is licensed under the MIT License - see the [LICENSE](https://github.com/monchin/tablers/blob/main/LICENSE) file for details.
