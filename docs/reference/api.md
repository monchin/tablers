# API Reference

This page provides detailed documentation for all public classes and functions in Tablers.

## Functions

### find_tables

Find all tables in a PDF page or from explicit edges.

```python
def find_tables(
    page: Page | None = None,
    extract_text: bool = True,
    clip: BBox | None = None,
    tf_settings: TfSettings | None = None,
    **kwargs: Unpack[TfSettingItems]
) -> list[Table]
```

**Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `page` | `Optional[Page]` | `None` | The PDF page to analyze. Can be `None` only if both strategies are `"explicit"` and `extract_text` is `False` |
| `extract_text` | `bool` | `True` | Whether to extract text content from table cells |
| `clip` | `Optional[BBox]` | `None` | Optional clip region (x1, y1, x2, y2). If provided, only edges within this region are used for table detection |
| `tf_settings` | `Optional[TfSettings]` | `None` | TableFinder settings object. If not provided, default settings are used |
| `**kwargs` |`Unpack[TfSettingItems]` | - | Additional keyword arguments passed to TfSettings |

**Returns:** `list[Table]` - A list of Table objects found in the page.

**Raises:**

- `ValueError` - If `page` is `None` and `extract_text` is `True`.
- `ValueError` - If `page` is `None` and either strategy is not `"explicit"`.

**Example:**

```python
from tablers import Document, find_tables

with Document("example.pdf") as doc:
    page = doc.get_page(0)
    tables = find_tables(page, extract_text=True)
    for table in tables:
        print(f"Table with {len(table.cells)} cells at {table.bbox}")
```

**Example with clip region:**

```python
from tablers import Document, find_tables

with Document("example.pdf") as doc:
    page = doc.get_page(0)
    # Only extract tables from a specific region
    clip = (100.0, 100.0, 400.0, 300.0)  # (x1, y1, x2, y2)
    tables = find_tables(page, extract_text=True, clip=clip)
```

**Example with explicit edges (no page required):**

```python
from tablers import Edge, TfSettings, find_tables

h_edges = [Edge("h", 0.0, 0.0, 100.0, 0.0), Edge("h", 0.0, 100.0, 100.0, 100.0)]
v_edges = [Edge("v", 0.0, 0.0, 0.0, 100.0), Edge("v", 100.0, 0.0, 100.0, 100.0)]

settings = TfSettings(
    horizontal_strategy="explicit",
    vertical_strategy="explicit",
    explicit_h_edges=h_edges,
    explicit_v_edges=v_edges,
)

tables = find_tables(page=None, extract_text=False, tf_settings=settings)
```

!!! warning "Clip coordinates on rotated pages"
    When a page is marked as rotated by 90° or 270°, `page.width` and `page.height` are defined based on the **upright orientation** (as you would normally view the page). However, all object coordinates (lines, text, etc.) within the PDF are defined based on the **unrotated coordinate system** (where `page.width` corresponds to the actual `page.height` after rotation is removed).

    Therefore, `clip` values must also be specified using the **unrotated coordinate system**. Failing to account for this may result in incorrect table extraction.

---

### find_all_cells_bboxes

Find all table cell bounding boxes in a PDF page or from explicit edges.

```python
def find_all_cells_bboxes(
    page: Page | None = None,
    clip: BBox | None = None,
    tf_settings: TfSettings | None = None,
    **kwargs: Unpack[TfSettingItems]
) -> list[tuple[float, float, float, float]]
```

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `page` | `Optional[Page]` | The PDF page to analyze. Can be `None` only if both strategies are `"explicit"` |
| `clip` | `Optional[BBox]` | Optional clip region (x1, y1, x2, y2). If provided, only edges within this region are used for cell detection |
| `tf_settings` | `Optional[TfSettings]` | TableFinder settings object |
| `**kwargs` |`Unpack[TfSettingItems]` | Additional keyword arguments passed to TfSettings |

**Returns:** `list[BBox]` - A list of bounding boxes (x1, y1, x2, y2) for each detected cell.

**Raises:** `RuntimeError` - If `page` is `None` and either strategy is not `"explicit"`.

**Example:**

```python
from tablers import Document, find_all_cells_bboxes

with Document("example.pdf") as doc:
    page = doc.get_page(0)
    cells = find_all_cells_bboxes(page)
    print(f"Found {len(cells)} cells")
```

**Example with clip region:**

```python
from tablers import Document, find_all_cells_bboxes

with Document("example.pdf") as doc:
    page = doc.get_page(0)
    # Only detect cells within a specific region
    clip = (100.0, 100.0, 400.0, 300.0)
    cells = find_all_cells_bboxes(page, clip=clip)
```

**Example with explicit edges (no page required):**

```python
from tablers import Edge, TfSettings, find_all_cells_bboxes

h_edges = [Edge("h", 0.0, 0.0, 100.0, 0.0), Edge("h", 0.0, 100.0, 100.0, 100.0)]
v_edges = [Edge("v", 0.0, 0.0, 0.0, 100.0), Edge("v", 100.0, 0.0, 100.0, 100.0)]

settings = TfSettings(
    horizontal_strategy="explicit",
    vertical_strategy="explicit",
    explicit_h_edges=h_edges,
    explicit_v_edges=v_edges,
)

cells = find_all_cells_bboxes(None, tf_settings=settings)
```

!!! warning "Clip coordinates on rotated pages"
    See the warning in [find_tables](#find_tables) about using clip with rotated pages.

---

### find_tables_from_cells

Construct tables from a list of cell bounding boxes.

```python
def find_tables_from_cells(
    cells: list[tuple[float, float, float, float]],
    extract_text: bool,
    page: Page | None = None,
    tf_settings: TfSettings | None = None,
    **kwargs: Unpack[TfSettingItems]
) -> list[Table]
```

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `cells` | `list[BBox]` | A list of cell bounding boxes to group into tables |
| `extract_text` | `bool` | Whether to extract text content from cells |
| `page` | `Optional[Page]` | The PDF page (required if extract_text is True) |
| `tf_settings` | `Optional[TfSettings]` | Table finder settings |
| `**kwargs` |`Unpack[TfSettingItems]` | Additional keyword arguments for settings |

**Returns:** `list[Table]` - A list of Table objects constructed from the cells.

**Raises:** `RuntimeError` - If extract_text is True but page is not provided.

---

### get_edges

Extract edges (lines and rectangle borders) from a PDF page or from explicit edges.

```python
def get_edges(
    page: Page | None = None,
    tf_settings: TfSettings | None = None,
    **kwargs: Unpack[TfSettingItems]
) -> dict[str, list[Edge]]
```

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `page` | `Optional[Page]` | The PDF page to extract edges from. Can be `None` only if both strategies are `"explicit"` |
| `tf_settings` | `Optional[TfSettings]` | TableFinder settings object |
| `**kwargs` | `Unpack[TfSettingItems]` | Additional keyword arguments passed to TfSettings |

**Returns:** `dict` - A dictionary with keys "h" (horizontal edges) and "v" (vertical edges).

**Raises:** `RuntimeError` - If `page` is `None` and either strategy is not `"explicit"`.

---

### plumber_edge_to_tablers_edge

Convert a pdfplumber edge dictionary to a Tablers `Edge` object.

```python
from tablers.edges import plumber_edge_to_tablers_edge

def plumber_edge_to_tablers_edge(
    plumber_edge: dict[str, Any],
    page_rotation: float,
    page_height: float,
    page_width: float,
) -> Edge
```

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `plumber_edge` | `dict[str, Any]` | A pdfplumber edge dictionary containing `orientation`, `x0`, `y0`, `x1`, `y1`, `linewidth`, and `stroking_color` |
| `page_rotation` | `float` | The rotation of the page in degrees |
| `page_height` | `float` | The height of the page |
| `page_width` | `float` | The width of the page |

**Returns:** `Edge` - A Tablers Edge object.

!!! tip
    This function can serve as a reference for writing conversion functions for other PDF libraries. See [Using Edges from Other Libraries](../usage/advanced.md#using-edges-from-other-libraries) for more details.

---

## Classes

### Document

Represents an opened PDF document.

```python
class Document:
    def __init__(
        self,
        path: Path | str | None = None,
        bytes: bytes | None = None,
        password: str | None = None
    )
```

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `path` | `Union[Path, str, None]` | File path to the PDF document |
| `bytes` | `Optional[bytes]` | PDF content as bytes |
| `password` | `Optional[str]` | Password for encrypted PDFs |

!!! note
    Either `path` or `bytes` must be provided, but not both. If both are provided, only `path` is used.

**Methods:**

| Method | Returns | Description |
|--------|---------|-------------|
| `page_count()` | `int` | Get the total number of pages |
| `get_page(page_num)` | `Page` | Retrieve a specific page by index (0-based) |
| `pages()` | `PageIterator` | Get an iterator over all pages |
| `close()` | `None` | Close the document and release resources |
| `is_closed()` | `bool` | Check if the document has been closed |

**Context Manager:**

```python
with Document("example.pdf") as doc:
    for page in doc:
        print(page.width, page.height)
```

---

### Page

Represents a single page in a PDF document.

**Attributes:**

| Attribute | Type | Description |
|-----------|------|-------------|
| `width` | `float` | The width of the page in points |
| `height` | `float` | The height of the page in points |
| `objects` | `Optional[Objects]` | Extracted objects, or None if not extracted |

**Methods:**

| Method | Returns | Description |
|--------|---------|-------------|
| `is_valid()` | `bool` | Check if the page reference is still valid |
| `extract_objects()` | `None` | Extract all objects from the page |
| `clear()` | `None` | Clear cached objects to free memory |

---

### Table

Represents a table extracted from a PDF page.

**Attributes:**

| Attribute | Type | Description |
|-----------|------|-------------|
| `bbox` | `tuple[float, float, float, float]` | Bounding box (x1, y1, x2, y2) |
| `cells` | `list[TableCell]` | All cells in the table |
| `rows` | `list[CellGroup]` | All rows in the table |
| `columns` | `list[CellGroup]` | All columns in the table |
| `page_index` | `int` | Index of the page containing this table |
| `text_extracted` | `bool` | Whether text has been extracted |

**Methods:**

| Method | Returns | Description |
|--------|---------|-------------|
| `to_csv()` | `str` | Convert to CSV format |
| `to_markdown()` | `str` | Convert to Markdown table format |
| `to_html()` | `str` | Convert to HTML table format |

!!! warning
    Export methods raise `ValueError` if text has not been extracted.

---

### TableCell

Represents a single cell in a table.

**Attributes:**

| Attribute | Type | Description |
|-----------|------|-------------|
| `bbox` | `tuple[float, float, float, float]` | Bounding box (x1, y1, x2, y2) |
| `text` | `str` | Text content of the cell |

---

### CellGroup

Represents a group of table cells arranged in a row or column.

**Attributes:**

| Attribute | Type | Description |
|-----------|------|-------------|
| `cells` | `list[Optional[TableCell]]` | Cells in this group, with None for empty positions |
| `bbox` | `tuple[float, float, float, float]` | Bounding box of the entire group |

---

### Objects

Container for all extracted objects from a PDF page.

**Attributes:**

| Attribute | Type | Description |
|-----------|------|-------------|
| `rects` | `list[Rect]` | All rectangles found in the page |
| `lines` | `list[Line]` | All line segments found in the page |
| `chars` | `list[Char]` | All text characters found in the page |

---

### Rect

Represents a rectangle extracted from a PDF page.

**Attributes:**

| Attribute | Type | Description |
|-----------|------|-------------|
| `bbox` | `tuple[float, float, float, float]` | Bounding box |
| `fill_color` | `tuple[int, int, int, int]` | Fill color (RGBA) |
| `stroke_color` | `tuple[int, int, int, int]` | Stroke color (RGBA) |
| `stroke_width` | `float` | Stroke width |

---

### Line

Represents a line segment extracted from a PDF page.

**Attributes:**

| Attribute | Type | Description |
|-----------|------|-------------|
| `line_type` | `Literal["straight", "curve"]` | Type of line |
| `points` | `list[tuple[float, float]]` | Points defining the line path |
| `color` | `tuple[int, int, int, int]` | Color (RGBA) |
| `width` | `float` | Line width |

---

### Char

Represents a text character extracted from a PDF page.

**Attributes:**

| Attribute | Type | Description |
|-----------|------|-------------|
| `unicode_char` | `Optional[str]` | Unicode character |
| `bbox` | `tuple[float, float, float, float]` | Bounding box |
| `rotation_degrees` | `float` | Clockwise rotation in degrees |
| `upright` | `bool` | Whether the character is upright |

---

### Edge

Represents a line edge extracted from a PDF page or created programmatically.

```python
class Edge:
    def __init__(
        self,
        orientation: Literal["h", "v"],
        x1: float,
        y1: float,
        x2: float,
        y2: float,
        width: float = 1.0,
        color: Color = (0, 0, 0, 255),
    ) -> None
```

**Constructor Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `orientation` | `Literal["h", "v"]` | - | "h" for horizontal, "v" for vertical |
| `x1` | `float` | - | Left x-coordinate |
| `y1` | `float` | - | Top y-coordinate |
| `x2` | `float` | - | Right x-coordinate |
| `y2` | `float` | - | Bottom y-coordinate |
| `width` | `float` | `1.0` | Stroke width |
| `color` | `Color` | `(0, 0, 0, 255)` | Stroke color (RGBA) |

**Raises:** `ValueError` - If `orientation` is not "h" or "v".

**Example:**

```python
from tablers import Edge

# Create a horizontal edge
h_edge = Edge("h", 0.0, 50.0, 100.0, 50.0)

# Create a vertical edge with custom width and color
v_edge = Edge("v", 50.0, 0.0, 50.0, 100.0, width=2.0, color=(255, 0, 0, 255))
```

**Attributes:**

| Attribute | Type | Description |
|-----------|------|-------------|
| `orientation` | `Literal["h", "v"]` | "h" for horizontal, "v" for vertical |
| `x1` | `float` | Left x-coordinate |
| `y1` | `float` | Top y-coordinate |
| `x2` | `float` | Right x-coordinate |
| `y2` | `float` | Bottom y-coordinate |
| `width` | `float` | Stroke width |
| `color` | `tuple[int, int, int, int]` | Stroke color (RGBA) |

---

## Type Aliases

| Alias | Definition | Description |
|-------|------------|-------------|
| `Point` | `tuple[float, float]` | A 2D point (x, y) |
| `BBox` | `tuple[float, float, float, float]` | Bounding box (x1, y1, x2, y2) |
| `Color` | `tuple[int, int, int, int]` | RGBA color (0-255 each) |
