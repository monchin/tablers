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
| `page` | `Optional[Page]` | The PDF page (required if `extract_text` is `True`) |
| `tf_settings` | `Optional[TfSettings]` | Table finder settings |
| `**kwargs` |`Unpack[TfSettingItems]` | Additional keyword arguments for settings |

**Returns:** `list[Table]` - A list of Table objects constructed from the cells.

**Raises:** `RuntimeError` - If `extract_text` is `True` but `page` is not provided.

!!! warning "Deprecated parameter `pdf_page`"
    The parameter was renamed from `pdf_page` to `page`. Passing `pdf_page` as a keyword argument still works but emits a `DeprecationWarning` and will be removed in a future release. Update your call sites:

    ```python
    # Old (deprecated)
    find_tables_from_cells(cells, extract_text=True, pdf_page=page)

    # New
    find_tables_from_cells(cells, extract_text=True, page=page)
    ```

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

### get_intersections_from_edges

Compute intersection points from a set of horizontal and vertical edges.

```python
def get_intersections_from_edges(
    h_edges: list[Edge],
    v_edges: list[Edge],
    tf_settings: TfSettings | None = None,
    **kwargs: Unpack[TfSettingItems]
) -> dict[tuple[float, float], dict[str, list[Edge]]]
```

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `h_edges` | `list[Edge]` | Horizontal edges (e.g. `edges["h"]` from `get_edges`) |
| `v_edges` | `list[Edge]` | Vertical edges (e.g. `edges["v"]` from `get_edges`) |
| `tf_settings` | `Optional[TfSettings]` | TableFinder settings object; controls intersection tolerances |
| `**kwargs` | `Unpack[TfSettingItems]` | Additional keyword arguments passed to TfSettings |

**Returns:** `dict` - A mapping from `(x, y)` intersection coordinates to a dict with keys `"h"` and `"v"`, each containing the list of edges that pass through that point.

!!! tip
    This function is designed to consume the output of `get_edges` directly:
    ```python
    edges = get_edges(page)
    intersections = get_intersections_from_edges(edges["h"], edges["v"])
    ```
    See [Inspecting Intersections](../usage/advanced.md#inspecting-intersections) for more details.

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

**Properties:**

| Property | Type | Description |
|----------|------|-------------|
| `page_count` | `int` | Total number of pages in the document |

**Methods:**

| Method | Returns | Description |
|--------|---------|-------------|
| `get_page(page_num)` | `Page` | Retrieve a specific page by index (0-based) |
| `pages()` | `Iterator[Page]` | Get a lazy iterator over all pages |
| `save_to_bytes()` | `bytes` | Serialize the document to bytes without encryption (see warning below) |
| `close()` | `None` | Close the document and release resources |
| `is_closed()` | `bool` | Check if the document has been closed |

!!! warning "`save_to_bytes()` strips encryption"
    If the original document was password-protected, `save_to_bytes()` returns a byte buffer that **can be opened without a password**. This is intentional—use it only when stripping the encryption is appropriate for your use case. The method also allocates a full in-memory copy of the PDF on every call; cache the result if you need it more than once.

**Context Manager:**

```python
with Document("example.pdf") as doc:
    for page in doc.pages():
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
| `page_idx` | `int` | Zero-based index of this page within its document |
| `rotation_degrees` | `float` | Clockwise rotation of the page in degrees |
| `objects` | `Optional[Objects]` | Extracted objects, or `None` if not yet extracted |
| `doc` | `Document` | The `Document` instance this page belongs to |

**Methods:**

| Method | Returns | Description |
|--------|---------|-------------|
| `is_valid()` | `bool` | Check if the page reference is still valid |
| `extract_objects()` | `None` | Extract all objects from the page |
| `clear_cache()` | `None` | Clear cached objects to free memory |
| `clear()` | `None` | Alias for `clear_cache()`; deprecated, prefer `clear_cache()` |

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
| `to_list()` | `list[list[TableCellValue]]` | Convert to list of rows; each cell has `text`, `merged_left`, and `merged_top` (see [TableCellValue](#tablecellvalue)) |

!!! warning
    Export methods raise `ValueError` if text has not been extracted.

---

### TableCellValue

One grid slot returned by `Table.to_list()`. Carries the cell text (when present) and merge direction so you can tell whether a merged slot continues from the left or from above.

**Attributes:**

| Attribute | Type | Description |
|-----------|------|-------------|
| `text` | `Optional[str]` | Cell text; `None` when this slot is merged (continuation of another cell) |
| `merged_left` | `bool` | `True` if this slot is merged with the cell to the left (same row) |
| `merged_top` | `bool` | `True` if this slot is merged with the cell above (same column) |

For a slot with content, `text` is set and both `merged_left` and `merged_top` are `False`. For a merged slot, `text` is `None` and at least one of `merged_left` or `merged_top` is `True`. When a cell spans both right and below, the bottom-right slot can have both `merged_left` and `merged_top` true.

**Repr:** `repr(cell)` returns a string `"(text, merged_left, merged_top)"`: `text` is shown as `None` or as a double-quoted string (internal quotes and backslashes are escaped); the two booleans are shown as `True` or `False`. Example: `("abc", False, False)` or `(None, True, False)`.

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
| `is_stroked` | `bool` | Whether the path is stroked |
| `fill_mode` | `FillMode` | Fill rule (NONE, WINDING, or EVEN_ODD) |

---

### FillMode

PDF path fill rule: winding (nonzero) or even-odd.

**Values:** `FillMode.NONE`, `FillMode.WINDING`, `FillMode.EVEN_ODD` (mirrors pdfium-render `PdfPathFillMode`)

---

### Line

Represents a line segment extracted from a PDF page.

**Attributes:**

| Attribute | Type | Description |
|-----------|------|-------------|
| `line_type` | `Literal["straight", "polyline", "curve"]` | Type of line |
| `points` | `list[tuple[float, float]]` | Points defining the line path |
| `stroke_color` | `tuple[int, int, int, int]` | Stroke color (RGBA) |
| `fill_color` | `tuple[int, int, int, int]` | Fill color (RGBA) |
| `width` | `float` | Line width |
| `is_stroked` | `bool` | Whether the line is stroked |
| `fill_mode` | `FillMode` | Fill rule (NONE, WINDING, or EVEN_ODD) |

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

---

## Debug Module (`tablers.debug`)

!!! note "Optional dependency"
    The debug module requires the `debug` extra. Install it with:
    ```bash
    pip install tablers[debug]
    ```

### PageImage

Renders a PDF page to a PIL image and provides drawing primitives for annotating detected tables, edges, and intersection points.

```python
from tablers.debug import PageImage

class PageImage:
    def __init__(
        self,
        page: Page,
        original: PIL.Image.Image | None = None,
        resolution: int | float = 72,
        antialias: bool = False,
    )
```

**Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `page` | `Page` | - | The page to render |
| `original` | `Optional[PIL.Image.Image]` | `None` | Pre-rendered image. If `None`, the page is rendered at the given resolution |
| `resolution` | `Union[int, float]` | `72` | Rendering resolution in DPI |
| `antialias` | `bool` | `False` | Enable anti-aliasing during rendering |

**Raises:** `RuntimeError` — If `original` is `None` and the document has already been closed.

!!! note "Password-protected PDFs"
    PageImage rendering supports only documents **without a password**. For password-protected PDFs, use `Document.save_to_bytes()` to obtain a decrypted copy, then open it with `Document(bytes=...)` and pass the resulting page to PageImage.

**Attributes:**

| Attribute | Type | Description |
|-----------|------|-------------|
| `original` | `PIL.Image.Image` | The unmodified rendered page image |
| `annotated` | `PIL.Image.Image` | The working copy with all annotations applied |
| `scale` | `float` | Ratio of image pixels to page points (`image_width / page_width`) |
| `bbox` | `BBox` | Page coordinate space: `(0, 0, page.width, page.height)` |
| `resolution` | `Union[int, float]` | The DPI used for rendering |

**Methods:**

| Method | Returns | Description |
|--------|---------|-------------|
| `reset()` | `PageImage` | Discard all annotations and restore `annotated` to `original` |
| `copy()` | `PageImage` | Return a new `PageImage` sharing the same `original` but with an independent `annotated` copy |
| `save(dest, format, quantize, colors, bits, **kwargs)` | `None` | Save the annotated image to a file path or `BytesIO` |
| `show()` | `None` | Display the annotated image (calls `PIL.Image.show`) |
| `_repr_png_()` | `bytes` | Return PNG bytes for Jupyter notebook inline display |

**Drawing methods** (all return `self` for chaining):

| Method | Description |
|--------|-------------|
| `draw_line(points, stroke, stroke_width)` | Draw a polyline. Accepts a tuple or list of two `(x, y)` points |
| `draw_lines(list_of_lines, stroke, stroke_width)` | Draw multiple lines |
| `draw_vline(location, stroke, stroke_width)` | Draw a vertical line spanning the full page height at `x = location` |
| `draw_vlines(locations, stroke, stroke_width)` | Draw multiple vertical lines |
| `draw_hline(location, stroke, stroke_width)` | Draw a horizontal line spanning the full page width at `y = location` |
| `draw_hlines(locations, stroke, stroke_width)` | Draw multiple horizontal lines |
| `draw_rect(bbox, fill, stroke, stroke_width)` | Draw a filled rectangle. Accepts a 4-tuple bbox `(x1, y1, x2, y2)` |
| `draw_rects(list_of_rects, fill, stroke, stroke_width)` | Draw multiple rectangles |
| `draw_circle(center, radius, fill, stroke)` | Draw a circle. Accepts a `(cx, cy)` center tuple |
| `draw_circles(list_of_circles, radius, fill, stroke)` | Draw multiple circles |
| `debug_table(table, fill, stroke, stroke_width)` | Draw a filled rectangle over every cell in a `Table` |
| `debug_tablefinder(tf_settings, **kwargs)` | Draw all detected tables (cell outlines) and detected edges |

**Color arguments** (`fill`, `stroke` in the methods above): accept either an RGBA tuple `(r, g, b, a)` or a string. String colors are resolved via PIL's [`ImageColor.getrgb`](https://pillow.readthedocs.io/en/stable/reference/ImageColor.html). For the list of supported string formats, see the [ImageColor reference](https://pillow.readthedocs.io/en/stable/reference/ImageColor.html). Alpha is set to 255 (opaque) for string colors; for transparency use an RGBA tuple.

**Default color constants** (importable from `tablers.debug`):

| Constant | Value | Description |
|----------|-------|-------------|
| `DEFAULT_FILL` | `(0, 0, 255, 50)` | Semi-transparent blue fill |
| `DEFAULT_STROKE` | `(255, 0, 0, 200)` | Near-opaque red stroke |
| `DEFAULT_STROKE_WIDTH` | `1` | Stroke width in pixels |
| `DEFAULT_RESOLUTION` | `72` | Default rendering DPI |

**Example — visualize table detection in Jupyter:**

```python
from tablers import Document
from tablers.debug import PageImage

with Document("example.pdf") as doc:
    page = doc.get_page(0)
    img = PageImage(page, resolution=150)

    # Draw tables, edges, and intersection points in one call
    img.debug_tablefinder()

    # Display inline (Jupyter auto-calls _repr_png_)
    img
```

**Example — annotate and save:**

```python
from tablers import Document, find_tables
from tablers.debug import PageImage

with Document("example.pdf") as doc:
    page = doc.get_page(0)
    tables = find_tables(page, extract_text=False)

    img = PageImage(page)
    for table in tables:
        img.debug_table(table)
    img.save("annotated.png", quantize=False)
```

**Example — method chaining:**

```python
img = (
    PageImage(page)
    .draw_hline(200.0)
    .draw_vline(300.0)
    .debug_tablefinder()
)
img.save("debug.png", quantize=False)
```
