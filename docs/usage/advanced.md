# Advanced Usage

This guide covers advanced features and customization options in Tablers.

## Custom Table Detection Settings

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

### Detection Strategies

Tablers supports four strategies for detecting table edges:

| Strategy | Description | Best For |
|----------|-------------|----------|
| `lines_strict` | Only uses explicit line objects | Tables with clear borders |
| `lines` | Uses lines and rectangle borders | Most common tables |
| `text` | Uses text alignment | Borderless tables |
| `explicit` | Uses only explicitly provided edges | Programmatic table creation |

```python
# For tables with clear borders
settings = TfSettings(
    vertical_strategy="lines_strict",
    horizontal_strategy="lines_strict"
)

# For tables without borders (text-based detection)
settings = TfSettings(
    vertical_strategy="text",
    horizontal_strategy="text",
    min_words_vertical=3,
    min_words_horizontal=1
)
```

## Custom Text Extraction Settings

Configure text extraction with `WordsExtractSettings`:

```python
from tablers import (
    Document, 
    find_tables_from_cells, 
    find_all_cells_bboxes, 
    WordsExtractSettings
)

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
        page=page,
        we_settings=we_settings
    )
```

### Text Extraction Options

| Option | Default | Description |
|--------|---------|-------------|
| `x_tolerance` | 3.0 | Horizontal tolerance for grouping characters into words |
| `y_tolerance` | 3.0 | Vertical tolerance for grouping characters into lines |
| `keep_blank_chars` | False | Whether to preserve whitespace characters |
| `use_text_flow` | False | Whether to use PDF's text flow order |
| `expand_ligatures` | True | Whether to expand ligatures (fi, fl, etc.) |
| `need_strip` | True | Whether to strip whitespace from cell text |

## Two-Step Table Extraction

For more control, separate cell detection from table construction:

```python
from tablers import Document, find_all_cells_bboxes, find_tables_from_cells

with Document("example.pdf") as doc:
    page = doc.get_page(0)

    # Step 1: Detect all cell bounding boxes
    cell_bboxes = find_all_cells_bboxes(page)
    print(f"Found {len(cell_bboxes)} cells")

    # Step 2: Optionally filter or modify cell_bboxes here
    # For example, filter out small cells
    filtered_cells = [
        bbox for bbox in cell_bboxes 
        if (bbox[2] - bbox[0]) > 10 and (bbox[3] - bbox[1]) > 10
    ]

    # Step 3: Construct tables from cells
    tables = find_tables_from_cells(
        filtered_cells,
        extract_text=True,
        page=page
    )
```

## Working with Edges

Extract and inspect edges directly for debugging or custom processing:

```python
from tablers import Document, get_edges

with Document("example.pdf") as doc:
    page = doc.get_page(0)
    edges = get_edges(page)

    print(f"Horizontal edges: {len(edges['h'])}")
    print(f"Vertical edges: {len(edges['v'])}")

    for edge in edges['h'][:5]:  # First 5 horizontal edges
        print(f"  ({edge.x1}, {edge.y1}) -> ({edge.x2}, {edge.y2})")
```

## Using Explicit Edges

Create tables programmatically by providing explicit edges instead of extracting from a PDF:

```python
from tablers import Edge, TfSettings, find_all_cells_bboxes, find_tables_from_cells

# Create edges for a 3x2 table grid
# Horizontal edges (3 lines for 2 rows)
h_edges = [
    Edge("h", 0.0, 0.0, 150.0, 0.0),     # Top border
    Edge("h", 0.0, 50.0, 150.0, 50.0),   # Middle line
    Edge("h", 0.0, 100.0, 150.0, 100.0), # Bottom border
]

# Vertical edges (4 lines for 3 columns)
v_edges = [
    Edge("v", 0.0, 0.0, 0.0, 100.0),     # Left border
    Edge("v", 50.0, 0.0, 50.0, 100.0),   # First divider
    Edge("v", 100.0, 0.0, 100.0, 100.0), # Second divider
    Edge("v", 150.0, 0.0, 150.0, 100.0), # Right border
]

settings = TfSettings(
    horizontal_strategy="explicit",
    vertical_strategy="explicit",
    explicit_h_edges=h_edges,
    explicit_v_edges=v_edges,
)

# Detect cells without requiring a PDF page
cells = find_all_cells_bboxes(None, tf_settings=settings)
print(f"Found {len(cells)} cells")  # Expected: 6 cells

# Build tables from cells (without text extraction)
tables = find_tables_from_cells(cells, extract_text=False)
```

### Combining Explicit Edges with PDF Edges

You can also combine explicit edges with PDF-extracted edges:

```python
from tablers import Document, Edge, TfSettings, find_tables

# Add custom horizontal dividers to existing PDF lines
custom_h_edges = [
    Edge("h", 50.0, 200.0, 400.0, 200.0),  # Extra horizontal line
]

settings = TfSettings(
    horizontal_strategy="lines",  # Use PDF lines for horizontal
    vertical_strategy="explicit", # Use only explicit vertical edges
    explicit_v_edges=[
        Edge("v", 100.0, 0.0, 100.0, 500.0),
        Edge("v", 300.0, 0.0, 300.0, 500.0),
    ],
)

with Document("example.pdf") as doc:
    page = doc.get_page(0)
    tables = find_tables(page, extract_text=True, tf_settings=settings)
```

### Creating Edge with Custom Style

```python
from tablers import Edge

# Edge with custom width and red color
edge = Edge(
    orientation="h",
    x1=0.0,
    y1=50.0,
    x2=100.0,
    y2=50.0,
    width=2.0,
    color=(255, 0, 0, 255),  # RGBA: Red
)
```

### Using Edges from Other Libraries

If you need to use edges generated by other PDF libraries for table extraction, simply write a conversion function to transform those edges into Tablers `Edge` objects, then use the `explicit` strategy.

Since [pdfplumber](https://github.com/jsvine/pdfplumber) is widely used, Tablers provides a built-in conversion function `plumber_edge_to_tablers_edge` in `tablers.edges`. This can also serve as an example for writing your own conversion function.

For other PDF libraries, you can write a similar conversion function following this pattern:

```python
from tablers import Edge

def your_library_edge_to_tablers_edge(lib_edge) -> Edge:
    """Convert edges from your library to tablers Edge objects."""
    # Extract orientation ("h" or "v")
    orientation = "h" if lib_edge.is_horizontal else "v"

    # Extract coordinates (adjust coordinate system if needed)
    x1, y1, x2, y2 = lib_edge.x1, lib_edge.y1, lib_edge.x2, lib_edge.y2

    # Extract styling (optional, defaults available)
    width = getattr(lib_edge, "width", 1.0)
    color = getattr(lib_edge, "color", (0, 0, 0, 255))

    return Edge(orientation, x1, y1, x2, y2, width, color)
```

## Working with Page Objects

Access raw page objects for custom processing:

```python
from tablers import Document

with Document("example.pdf") as doc:
    page = doc.get_page(0)

    # Extract objects (chars, lines, rects)
    page.extract_objects()

    if page.objects:
        print(f"Characters: {len(page.objects.chars)}")
        print(f"Lines: {len(page.objects.lines)}")
        print(f"Rectangles: {len(page.objects.rects)}")

        # Access individual characters
        for char in page.objects.chars[:10]:
            print(f"  '{char.unicode_char}' at {char.bbox}")

    # Clear cached objects to free memory
    page.clear()
```

## Tolerance Settings

Tablers provides various tolerance settings for fine-tuning detection:

### Snapping Tolerances

Control how edges are snapped together:

```python
settings = TfSettings(
    snap_x_tolerance=5.0,  # Snap vertical edges within 5 points
    snap_y_tolerance=5.0,  # Snap horizontal edges within 5 points
)
```

### Joining Tolerances

Control how edge segments are joined:

```python
settings = TfSettings(
    join_x_tolerance=3.0,  # Join horizontal segments within 3 points
    join_y_tolerance=3.0,  # Join vertical segments within 3 points
)
```

### Intersection Tolerances

Control how edge intersections are detected:

```python
settings = TfSettings(
    intersection_x_tolerance=3.0,
    intersection_y_tolerance=3.0,
)
```

## Performance Tips

### Memory Efficiency

For large PDFs, process pages one at a time:

```python
from tablers import Document, find_tables

with Document("large_file.pdf") as doc:
    for page in doc.pages():
        tables = find_tables(page, extract_text=True)
        # Process tables immediately
        for table in tables:
            process_table(table)
        # Page is released when loop continues
```

### Skip Text Extraction

If you only need table structure, skip text extraction:

```python
# Faster when you only need cell positions
tables = find_tables(page, extract_text=False)

for table in tables:
    print(f"Table at {table.bbox}")
    for cell in table.cells:
        print(f"  Cell at {cell.bbox}")
        # cell.text will be empty
```

### Prefilter Edges

Reduce noise by setting minimum edge length:

```python
settings = TfSettings(
    edge_min_length=10.0,           # Final minimum edge length
    edge_min_length_prefilter=5.0,  # Initial filtering before merge
)
```

## Error Handling

```python
from tablers import Document, find_tables

try:
    doc = Document("example.pdf")
except Exception as e:
    print(f"Failed to open document: {e}")
    raise

try:
    with doc:
        page = doc.get_page(100)  # May raise IndexError
except IndexError:
    print("Page index out of range")
except RuntimeError as e:
    print(f"Runtime error: {e}")
```

## Next Steps

- See [Settings Reference](../reference/settings.md) for all configuration options
- Check the [API Reference](../reference/api.md) for complete documentation
