# API Reference

This page provides detailed documentation for all public classes and functions in Tablers.

## Functions

### find_tables

Find all tables in a PDF page.

```python
def find_tables(
    page: Page,
    extract_text: bool,
    tf_settings: TfSettings | None = None,
    **kwargs: Unpack[TfSettingItems]
) -> list[Table]
```

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `page` | `Page` | The PDF page to analyze |
| `extract_text` | `bool` | Whether to extract text content from table cells |
| `tf_settings` | `Optional[TfSettings]` | TableFinder settings object. If not provided, default settings are used |
| `**kwargs` |`Unpack[TfSettingItems]` | Additional keyword arguments passed to TfSettings |

**Returns:** `list[Table]` - A list of Table objects found in the page.

**Example:**

```python
from tablers import Document, find_tables

with Document("example.pdf") as doc:
    page = doc.get_page(0)
    tables = find_tables(page, extract_text=True)
    for table in tables:
        print(f"Table with {len(table.cells)} cells at {table.bbox}")
```

---

### find_all_cells_bboxes

Find all table cell bounding boxes in a PDF page.

```python
def find_all_cells_bboxes(
    page: Page,
    tf_settings: TfSettings | None = None,
    **kwargs: Unpack[TfSettingItems]
) -> list[tuple[float, float, float, float]]
```

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `page` | `Page` | The PDF page to analyze |
| `tf_settings` | `Optional[TfSettings]` | TableFinder settings object |
| `**kwargs` |`Unpack[TfSettingItems]` | Additional keyword arguments passed to TfSettings |

**Returns:** `list[BBox]` - A list of bounding boxes (x1, y1, x2, y2) for each detected cell.

**Example:**

```python
from tablers import Document, find_all_cells_bboxes

with Document("example.pdf") as doc:
    page = doc.get_page(0)
    cells = find_all_cells_bboxes(page)
    print(f"Found {len(cells)} cells")
```

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

Extract edges (lines and rectangle borders) from a PDF page.

```python
def get_edges(
    page: Page,
    settings: dict | None = None
) -> dict[str, list[Edge]]
```

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `page` | `Page` | The PDF page to extract edges from |
| `settings` | `dict \| None` | Dictionary of settings for edge extraction |

**Returns:** `dict` - A dictionary with keys "h" (horizontal edges) and "v" (vertical edges).

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
| `objects` | `Objects \| None` | Extracted objects, or None if not extracted |

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
| `cells` | `list[TableCell \| None]` | Cells in this group, with None for empty positions |
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

Represents a line edge extracted from a PDF page.

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

