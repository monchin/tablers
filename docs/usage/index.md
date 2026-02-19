# Basic Usage

This guide covers the fundamental concepts and common use cases for Tablers.

## Core Concepts

### Document

The `Document` class is the entry point for working with PDF files. It manages the PDF file handle and provides access to pages.

```python
from tablers import Document

# Open from file path
doc = Document("example.pdf")

# Open from bytes
with open("example.pdf", "rb") as f:
    doc = Document(bytes=f.read())

# Open encrypted PDF
doc = Document("encrypted.pdf", password="secret")
```

### Page

A `Page` represents a single page in the PDF document. You can access pages by index or iterate through all pages.

```python
from tablers import Document

with Document("example.pdf") as doc:
    # Get page count (property, no parentheses)
    print(f"Total pages: {doc.page_count}")

    # Get specific page (0-indexed)
    page = doc.get_page(0)
    print(f"Page size: {page.width} x {page.height}")
    print(f"Page index: {page.page_idx}")

    # Iterate through all pages
    for page in doc.pages():
        print(f"Processing page {page.page_idx}...")
```

### Table

A `Table` object represents an extracted table and contains cells organized into rows and columns.

```python
from tablers import Document, find_tables

with Document("example.pdf") as doc:
    page = doc.get_page(0)
    tables = find_tables(page, extract_text=True)

    for table in tables:
        # Access table properties
        print(f"Bounding box: {table.bbox}")
        print(f"Number of cells: {len(table.cells)}")
        print(f"Number of rows: {len(table.rows)}")
        print(f"Number of columns: {len(table.columns)}")
        print(f"Page index: {table.page_index}")
```

### TableCell

Each `TableCell` contains the text content and bounding box of a single cell.

```python
for table in tables:
    for cell in table.cells:
        print(f"Text: {cell.text}")
        print(f"Position: {cell.bbox}")  # (x1, y1, x2, y2)
```

## Finding Tables

### Basic Table Detection

Use `find_tables()` to detect and extract tables from a page:

```python
from tablers import Document, find_tables

with Document("example.pdf") as doc:
    page = doc.get_page(0)

    # Extract tables with text
    tables = find_tables(page, extract_text=True)

    # Extract tables without text (faster)
    tables = find_tables(page, extract_text=False)
```

### Filtering Tables

Filter tables by size to exclude small or irrelevant results:

```python
from tablers import Document, find_tables

with Document("example.pdf") as doc:
    page = doc.get_page(0)

    # Only get tables with at least 2 rows and 3 columns
    tables = find_tables(
        page,
        extract_text=True,
        min_rows=2,
        min_columns=3,
        include_single_cell=False  # Exclude single-cell tables (default)
    )
```

### Extracting Tables from a Specific Region

Use the `clip` parameter to extract tables only from a specific area of the page:

```python
from tablers import Document, find_tables

with Document("example.pdf") as doc:
    page = doc.get_page(0)

    # Define a clip region (x1, y1, x2, y2)
    # Only tables within this region will be extracted
    clip = (100.0, 100.0, 400.0, 300.0)

    tables = find_tables(page, extract_text=True, clip=clip)
```

!!! warning "Clip coordinates on rotated pages"
    When a page is marked as rotated by 90° or 270°, `page.width` and `page.height` are defined based on the **upright orientation** (as you would normally view the page). However, all object coordinates (lines, text, etc.) within the PDF are defined based on the **unrotated coordinate system** (where `page.width` corresponds to the actual `page.height` after rotation is removed).

    Therefore, `clip` values must also be specified using the **unrotated coordinate system**. Failing to account for this may result in incorrect table extraction.

    For example, if a page is rotated 90° clockwise and has `page.width=595` and `page.height=842` (A4 portrait dimensions), the actual coordinate system for objects is 842×595. To clip the top-left corner of the visual page, you would need to account for this transformation.

## Working with Rows and Columns

### Accessing Rows

```python
for table in tables:
    for row in table.rows:
        print(f"Row bbox: {row.bbox}")
        for cell in row.cells:
            if cell is not None:
                print(f"  Cell: {cell.text}")
            else:
                print(f"  Empty cell")
```

### Accessing Columns

```python
for table in tables:
    for col in table.columns:
        print(f"Column bbox: {col.bbox}")
        for cell in col.cells:
            if cell is not None:
                print(f"  Cell: {cell.text}")
```

## Exporting Tables

### Export to CSV

```python
csv_content = table.to_csv()
print(csv_content)

# Save to file
with open("output.csv", "w", encoding="utf-8") as f:
    f.write(csv_content)

# If you need to use pandas or polars, you can transform the table manually. 
# Remember to install the required dependencies because tablers does not depend on them.
import pandas as pd
from io import StringIO
df_pd = pd.read_csv(StringIO(csv_content))

import polars as pl
df_pl = pl.read_csv(StringIO(csv_content))
```

### Export to Markdown

```python
md_content = table.to_markdown()
print(md_content)
```

Output example:

```markdown
| Header1 | Header2 | Header3 |
| --- | --- | --- |
| Cell1 | Cell2 | Cell3 |
| Cell4 | Cell5 | Cell6 |
```

### Export to HTML

```python
html_content = table.to_html()
print(html_content)

# Save to file
with open("output.html", "w", encoding="utf-8") as f:
    f.write(html_content)
```

## Saving a Document to Bytes

Use `save_to_bytes()` to serialize the current document into an in-memory byte buffer. This is useful for passing the PDF to another library, uploading it, or storing it without writing to disk.

```python
from tablers import Document

with Document("example.pdf") as doc:
    data: bytes = doc.save_to_bytes()
    # data is a valid PDF byte stream
    with open("copy.pdf", "wb") as f:
        f.write(data)
```

### Decrypting an Encrypted PDF

`save_to_bytes()` always produces an **unencrypted** output. If the original PDF was password-protected, the returned bytes can be opened without any password:

```python
from tablers import Document

with Document("encrypted.pdf", password="secret") as doc:
    decrypted: bytes = doc.save_to_bytes()

# Re-open without password
with Document(bytes=decrypted) as doc2:
    print(doc2.page_count)
```

!!! warning
    `save_to_bytes()` **strips the encryption** from the document. Only use this when you intentionally want to remove password protection. The method also allocates a full in-memory copy of the PDF on each call—cache the result if you need it more than once.

## Resource Management

Always close documents when done to release resources:

```python
# Using context manager (recommended)
with Document("example.pdf") as doc:
    # Process document
    pass
# Document is automatically closed

# Manual management
doc = Document("example.pdf")
try:
    # Process document
    pass
finally:
    doc.close()

# Check if document is closed
print(doc.is_closed())
```

## Next Steps

- Learn about [Advanced Usage](advanced.md) for custom settings
- Explore the [API Reference](../reference/api.md) for complete documentation
