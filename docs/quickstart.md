# Quick Start

This guide will help you get started with Tablers quickly.

## Basic Table Extraction

The simplest way to extract tables from a PDF:

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

## Using Context Manager

The recommended approach is to use a context manager for automatic resource cleanup:

```python
from tablers import Document, find_tables

with Document("example.pdf") as doc:
    page = doc.get_page(0)  # Get first page
    tables = find_tables(page, extract_text=True)
    
    for table in tables:
        print(f"Table bbox: {table.bbox}")
```

## Opening PDF from Bytes

You can also open PDFs from bytes in memory:

```python
from tablers import Document, find_tables

with open("example.pdf", "rb") as f:
    pdf_bytes = f.read()

doc = Document(bytes=pdf_bytes)
# ... process document
doc.close()
```

## Working with Encrypted PDFs

Tablers supports password-protected PDF documents:

```python
from tablers import Document

doc = Document("encrypted.pdf", password="secret123")
# ... process document
doc.close()
```

## Exporting Tables

Export tables to various formats:

```python
from tablers import Document, find_tables

with Document("example.pdf") as doc:
    page = doc.get_page(0)
    tables = find_tables(page, extract_text=True)
    
    for i, table in enumerate(tables):
        # Export to CSV
        csv_content = table.to_csv()
        with open(f"table_{i}.csv", "w") as f:
            f.write(csv_content)
        
        # Export to Markdown
        md_content = table.to_markdown()
        print(md_content)
        
        # Export to HTML
        html_content = table.to_html()
        with open(f"table_{i}.html", "w") as f:
            f.write(html_content)
```

## Processing Multiple Pages

Process all pages in a document efficiently:

```python
from tablers import Document, find_tables

with Document("multi_page.pdf") as doc:
    print(f"Processing {doc.page_count()} pages")
    
    all_tables = []
    for page in doc.pages():
        tables = find_tables(page, extract_text=True)
        all_tables.extend(tables)
    
    print(f"Found {len(all_tables)} tables in total")
```

## Next Steps

- Learn about [Advanced Usage](usage/advanced.md) for fine-tuning table detection
- Explore the [API Reference](reference/api.md) for complete documentation
- Check [Settings Reference](reference/settings.md) for all configuration options


