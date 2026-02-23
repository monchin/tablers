# Settings Reference

This page documents all configuration settings available in Tablers.

## TfSettings

Table finder settings control how tables are detected and extracted.

```python
from tablers import TfSettings

settings = TfSettings(
    vertical_strategy="lines_strict",
    horizontal_strategy="lines_strict",
    snap_x_tolerance=3.0,
    # ... other options
)
```

### Detection Strategy

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `vertical_strategy` | `Literal["lines", "lines_strict", "text", "explicit"]` | `"lines_strict"` | Strategy for detecting vertical edges |
| `horizontal_strategy` | `Literal["lines", "lines_strict", "text", "explicit"]` | `"lines_strict"` | Strategy for detecting horizontal edges |

**Strategy Options:**

- `"lines_strict"` - Only uses explicit line objects. Best for tables with clear borders.
- `"lines"` - Uses lines and rectangle borders. Good for most common tables.
- `"text"` - Uses text alignment to infer edges. Best for borderless tables.
- `"explicit"` - Uses only explicitly provided edges via `explicit_h_edges` and `explicit_v_edges`. Best for programmatic table creation.

### Tolerance Settings

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `snap_x_tolerance` | `float` | `3.0` | Tolerance for snapping vertical edges together |
| `snap_y_tolerance` | `float` | `3.0` | Tolerance for snapping horizontal edges together |
| `join_x_tolerance` | `float` | `3.0` | Tolerance for joining horizontal edge segments |
| `join_y_tolerance` | `float` | `3.0` | Tolerance for joining vertical edge segments |
| `intersection_x_tolerance` | `float` | `3.0` | X-tolerance for detecting edge intersections |
| `intersection_y_tolerance` | `float` | `3.0` | Y-tolerance for detecting edge intersections |

### Edge Detection

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `edge_min_length` | `float` | `3.0` | Minimum length for edges to be included in final detection |
| `edge_min_length_prefilter` | `float` | `1.0` | Minimum length for edges before merging operations |
| `min_words_vertical` | `int` | `3` | Minimum words required for vertical text-based edge detection |
| `min_words_horizontal` | `int` | `1` | Minimum words required for horizontal text-based edge detection |
| `exclude_white_edges` | `bool` | `True` | Whether to exclude white edges (RGB = 255, 255, 255) from table detection |

### Explicit Edges

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `explicit_h_edges` | `Optional[list[Edge]]` | `None` | Explicit horizontal edges to include in table detection |
| `explicit_v_edges` | `Optional[list[Edge]]` | `None` | Explicit vertical edges to include in table detection |

When using `"explicit"` strategy, you must provide edges via these parameters. This allows programmatic table creation without requiring a PDF page:

```python
from tablers import Edge, TfSettings, find_all_cells_bboxes

# Create edges for a 2x2 grid
h_edges = [
    Edge("h", 0.0, 0.0, 100.0, 0.0),
    Edge("h", 0.0, 50.0, 100.0, 50.0),
    Edge("h", 0.0, 100.0, 100.0, 100.0),
]
v_edges = [
    Edge("v", 0.0, 0.0, 0.0, 100.0),
    Edge("v", 50.0, 0.0, 50.0, 100.0),
    Edge("v", 100.0, 0.0, 100.0, 100.0),
]

settings = TfSettings(
    horizontal_strategy="explicit",
    vertical_strategy="explicit",
    explicit_h_edges=h_edges,
    explicit_v_edges=v_edges,
)

# No page required when both strategies are explicit
cells = find_all_cells_bboxes(None, tf_settings=settings)
```

### Table Filtering

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `include_single_cell` | `bool` | `False` | Whether to include tables with only a single cell |
| `min_rows` | `Optional[int]` | `None` | Minimum number of rows required. `None` means no filtering |
| `min_columns` | `Optional[int]` | `None` | Minimum number of columns required. `None` means no filtering |

### Text Extraction (within TfSettings)

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `text_x_tolerance` | `float` | `3.0` | X-tolerance for text extraction |
| `text_y_tolerance` | `float` | `3.0` | Y-tolerance for text extraction |
| `text_keep_blank_chars` | `bool` | `False` | Whether to keep blank characters |
| `text_use_text_flow` | `bool` | `False` | Whether to use PDF text flow order |
| `text_read_in_clockwise` | `bool` | `True` | Whether text reads in clockwise direction |
| `text_split_at_punctuation` | `Union[Literal["all"], str, None` | `None` | Punctuation splitting configuration |
| `text_expand_ligatures` | `bool` | `True` | Whether to expand ligatures |
| `text_need_strip` | `bool` | `True` | Whether to strip whitespace from cell text |

### Complete Example

```python
from tablers import TfSettings

settings = TfSettings(
    # Detection strategy
    vertical_strategy="lines",
    horizontal_strategy="lines",

    # Tolerance settings
    snap_x_tolerance=5.0,
    snap_y_tolerance=5.0,
    join_x_tolerance=3.0,
    join_y_tolerance=3.0,
    intersection_x_tolerance=3.0,
    intersection_y_tolerance=3.0,

    # Edge detection
    edge_min_length=10.0,
    edge_min_length_prefilter=5.0,
    min_words_vertical=3,
    min_words_horizontal=1,
    exclude_white_edges=True,

    # Table filtering
    include_single_cell=False,
    min_rows=2,
    min_columns=2,

    # Text extraction
    text_x_tolerance=3.0,
    text_y_tolerance=3.0,
    text_need_strip=True,
)
```

---

## WordsExtractSettings

Settings for text/word extraction from PDF pages.

```python
from tablers import WordsExtractSettings

we_settings = WordsExtractSettings(
    x_tolerance=3.0,
    y_tolerance=3.0,
)
```

### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `x_tolerance` | `float` | `3.0` | Horizontal tolerance for grouping characters into words |
| `y_tolerance` | `float` | `3.0` | Vertical tolerance for grouping characters into lines |
| `keep_blank_chars` | `bool` | `False` | Whether to preserve blank/whitespace characters |
| `use_text_flow` | `bool` | `False` | Whether to use the PDF's text flow order |
| `text_read_in_clockwise` | `bool` | `True` | Whether text reads in clockwise direction |
| `split_at_punctuation` | `Union[Literal["all"], str, None]` | `None` | Punctuation splitting configuration |
| `expand_ligatures` | `bool` | `True` | Whether to expand ligatures into individual characters |
| `need_strip` | `bool` | `True` | Whether to strip leading/trailing whitespace from cell text |

### Punctuation Splitting

The `split_at_punctuation` parameter controls how text is split at punctuation:

- `None` - No splitting at punctuation
- `"all"` - Split at all punctuation characters
- `str` - Split at specific characters (e.g., `".,;:"`)

### Complete Example

```python
from tablers import WordsExtractSettings

we_settings = WordsExtractSettings(
    x_tolerance=3.0,
    y_tolerance=3.0,
    keep_blank_chars=False,
    use_text_flow=False,
    text_read_in_clockwise=True,
    split_at_punctuation=None,
    expand_ligatures=True,
    need_strip=True,
)
```

---

## Using Settings with Functions

### With find_tables

```python
from tablers import Document, find_tables, TfSettings

settings = TfSettings(
    vertical_strategy="lines",
    min_rows=2,
)

with Document("example.pdf") as doc:
    page = doc.get_page(0)
    tables = find_tables(page, extract_text=True, tf_settings=settings)
```

### With Keyword Arguments

You can also pass settings as keyword arguments directly:

```python
from tablers import Document, find_tables

with Document("example.pdf") as doc:
    page = doc.get_page(0)
    tables = find_tables(
        page,
        extract_text=True,
        vertical_strategy="lines",
        horizontal_strategy="lines",
        min_rows=2,
        snap_x_tolerance=5.0,
    )
```

### With find_tables_from_cells

```python
from tablers import (
    Document,
    find_all_cells_bboxes,
    find_tables_from_cells,
    WordsExtractSettings
)

we_settings = WordsExtractSettings(
    x_tolerance=5.0,
    y_tolerance=5.0,
)

with Document("example.pdf") as doc:
    page = doc.get_page(0)
    cells = find_all_cells_bboxes(page)
    tables = find_tables_from_cells(
        cells,
        extract_text=True,
        page=page,
        we_settings=we_settings,
    )
```
