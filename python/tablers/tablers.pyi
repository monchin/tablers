"""
Type stubs for the tablers Rust extension module.

This module provides type hints for the PDF table extraction library
implemented in Rust and exposed to Python via PyO3.
"""

from __future__ import annotations

import sys
from collections.abc import Iterator
from pathlib import Path
from types import TracebackType
from typing import Annotated, Literal, TypeAlias, TypedDict

if sys.version_info < (3, 11):
    from typing_extensions import Self, Unpack
else:
    from typing import Self, Unpack

def validate_non_negative(value: int | float) -> bool:
    return value >= 0

NonNegativeFloat: TypeAlias = Annotated[float, validate_non_negative]
"""A non-negative floating point number."""

NonNegativeInt: TypeAlias = Annotated[int, validate_non_negative]
"""A non-negative integer."""

Point: TypeAlias = tuple[float, float]
"""A 2D point represented as (x, y) coordinates."""

BBox: TypeAlias = tuple[float, float, float, float]
"""A bounding box represented as (x1, y1, x2, y2) coordinates."""

Color: TypeAlias = tuple[int, int, int, int]
"""An RGBA color tuple, each component in range 0-255."""

__version__: str
"""The version string of the tablers library."""

class PdfiumRuntime:
    """
    A wrapper around the Pdfium library runtime.

    This class holds the Pdfium instance and provides the foundation
    for opening and working with PDF documents.

    If the library has already been initialized, creating a new PdfiumRuntime
    will reuse the existing instance and ignore the provided path.

    Parameters
    ----------
    dll_path : Path or str
        The file path to the Pdfium dynamic library (.dll, .so, or .dylib).
        This is only used on the first initialization; subsequent calls
        will reuse the existing instance.

    Notes
    -----
    This class is typically not used directly. Use `get_runtime()` to obtain
    a runtime instance, which handles initialization automatically.
    """

    def __init__(self, dll_path: Path | str): ...
    @staticmethod
    def is_initialized() -> bool:
        """
        Check if the Pdfium library has been initialized.

        Returns
        -------
        bool
            True if the library has been initialized, False otherwise.
        """
        ...

class PageIterator(Iterator[Page]):
    """
    Iterator over PDF pages.

    This iterator is memory-efficient for large PDFs as it loads
    pages on demand rather than all at once.

    Yields
    ------
    Page
        The next page in the document.
    """

    def __iter__(self) -> PageIterator: ...
    def __next__(self) -> Page: ...

class Document:
    """
    Represents an opened PDF document.

    Provides methods to access pages and metadata of a PDF document.
    The document can be closed explicitly, after which all operations will fail.

    Parameters
    ----------
    pdfium_rt : PdfiumRuntime
        The Pdfium runtime instance to use.
    path : Path or str, optional
        File path to the PDF document.
    bytes : bytes, optional
        PDF content as bytes.
    password : str, optional
        Password for encrypted PDFs.

    Notes
    -----
    Either `path` or `bytes` must be provided, but not both.
    """

    def __init__(
        self,
        pdfium_rt: PdfiumRuntime,
        path: Path | str | None = None,
        bytes: bytes | None = None,
        password: str | None = None,
    ): ...
    def page_count(self) -> int:
        """
        Get the total number of pages in the document.

        Returns
        -------
        int
            The number of pages in the document.

        Raises
        ------
        RuntimeError
            If the document has been closed.
        """
        ...

    def get_page(self, page_num: int) -> Page:
        """
        Retrieve a specific page by index.

        Parameters
        ----------
        page_num : int
            The zero-based index of the page to retrieve.

        Returns
        -------
        Page
            The requested page.

        Raises
        ------
        IndexError
            If the page index is out of range.
        RuntimeError
            If the document has been closed.
        """
        ...

    def pages(self) -> PageIterator:
        """
        Get an iterator over all pages in the document.

        Returns
        -------
        PageIterator
            An iterator that yields pages one at a time.
        """
        ...

    def close(self) -> None:
        """
        Close the document and release resources.

        After calling this method, all Page objects from this document
        become invalid.
        """
        ...

    def is_closed(self) -> bool:
        """
        Check if the document has been closed.

        Returns
        -------
        bool
            True if the document is closed, False otherwise.
        """
        ...

    def __iter__(self) -> PageIterator: ...
    def __enter__(self) -> Self:
        """
        Context manager entry point.

        Returns
        -------
        Self
            The document instance for use in `with` statements.

        Examples
        --------
        >>> with Document(runtime, path="example.pdf") as doc:
        ...     for page in doc:
        ...         print(page.width, page.height)
        """
        ...

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_val: BaseException | None,
        exc_tb: TracebackType | None,
    ) -> bool:
        """
        Context manager exit point.

        Closes the document when exiting the `with` block, regardless
        of whether an exception occurred.

        Parameters
        ----------
        exc_type : type[BaseException] or None
            The exception type, if an exception was raised.
        exc_val : BaseException or None
            The exception value, if an exception was raised.
        exc_tb : TracebackType or None
            The exception traceback, if an exception was raised.

        Returns
        -------
        bool
            False, indicating that exceptions are not suppressed.
        """
        ...

class Page:
    """
    Represents a single page in a PDF document.

    Provides access to page properties like dimensions and rotation,
    as well as methods to extract objects and text from the page.

    Attributes
    ----------
    width : float
        The width of the page in points.
    height : float
        The height of the page in points.
    """

    width: float
    height: float

    def is_valid(self) -> bool:
        """
        Check if the page reference is still valid.

        Returns
        -------
        bool
            True if the page is valid (document not closed), False otherwise.
        """
        ...

    def extract_objects(self) -> None:
        """
        Extract all objects (characters, lines, rectangles) from the page.

        This method caches the extracted objects for subsequent access
        via the `objects` property.
        """
        ...

    def clear(self):
        """
        Clear the cached objects to free memory.
        """
        ...

    @property
    def objects(self) -> Objects | None:
        """
        Get the extracted objects from the page.

        Returns
        -------
        Objects or None
            An Objects instance containing all extracted objects,
            or None if objects have not been extracted yet.
        """
        ...

class Objects:
    """
    Container for all extracted objects from a PDF page.

    Attributes
    ----------
    rects : list of Rect
        All rectangles found in the page.
    lines : list of Line
        All line segments found in the page.
    chars : list of Char
        All text characters found in the page.
    """

    rects: list[Rect]
    lines: list[Line]
    chars: list[Char]

class Rect:
    """
    Represents a rectangle extracted from a PDF page.

    Rectangles are typically used as table cell borders or backgrounds.

    Attributes
    ----------
    bbox : BBox
        The bounding box of the rectangle (x1, y1, x2, y2).
    fill_color : Color
        The fill color as an RGBA tuple.
    stroke_color : Color
        The stroke (border) color as an RGBA tuple.
    stroke_width : float
        The stroke width of the rectangle border.
    """

    bbox: BBox
    fill_color: Color
    stroke_color: Color
    stroke_width: float

class Line:
    """
    Represents a line segment extracted from a PDF page.

    Lines can be straight or curved and are commonly used for table borders.

    Attributes
    ----------
    line_type : {"straight", "curve"}
        The type of line segment.
    points : list of Point
        The points that define the line path.
    color : Color
        The color of the line as an RGBA tuple.
    width : float
        The width of the line stroke.
    """

    line_type: Literal["straight", "curve"]
    points: list[Point]
    color: Color
    width: float

class Char:
    """
    Represents a text character extracted from a PDF page.

    Each character includes its Unicode value, position, and rotation information.

    Attributes
    ----------
    unicode_char : str or None
        The Unicode string representation of the character.
    bbox : BBox
        The bounding box of the character (x1, y1, x2, y2).
    rotation_degrees : float
        The clockwise rotation of the character in degrees.
    upright : bool
        Whether the character is upright (horizontal text).
    """

    unicode_char: str | None
    bbox: BBox
    rotation_degrees: float
    upright: bool

class Edge:
    """
    Represents a line edge extracted from a PDF page.

    An edge can be either horizontal or vertical and is used
    for table structure detection.

    Attributes
    ----------
    orientation : {"h", "v"}
        The orientation of the edge ("h" for horizontal, "v" for vertical).
    x1 : float
        The left x-coordinate of the edge.
    y1 : float
        The top y-coordinate of the edge.
    x2 : float
        The right x-coordinate of the edge.
    y2 : float
        The bottom y-coordinate of the edge.
    width : float
        The stroke width of the edge.
    color : Color
        The stroke color as an RGBA tuple.
    """

    orientation: Literal["h", "v"]
    x1: float
    y1: float
    x2: float
    y2: float
    width: float
    color: Color

    def __init__(
        self,
        orientation: Literal["h", "v"],
        x1: float,
        y1: float,
        x2: float,
        y2: float,
        width: float = 1.0,
        color: Color = (0, 0, 0, 255),
    ) -> None:
        """
        Creates a new Edge.

        Parameters
        ----------
        orientation : {"h", "v"}
            The orientation of the edge ("h" for horizontal, "v" for vertical).
        x1 : float
            The left x-coordinate of the edge.
        y1 : float
            The top y-coordinate of the edge.
        x2 : float
            The right x-coordinate of the edge.
        y2 : float
            The bottom y-coordinate of the edge.
        width : float, optional
            The stroke width of the edge (default: 1.0).
        color : Color, optional
            The stroke color as an RGBA tuple (default: (0, 0, 0, 255)).
        """
        ...

class TableCell:
    """
    Represents a single cell in a table.

    Attributes
    ----------
    bbox : BBox
        The bounding box of the cell (x1, y1, x2, y2).
    text : str
        The text content of the cell.
    """

    bbox: BBox
    text: str

class CellGroup:
    """
    Represents a group of table cells arranged in a row or column.

    Cells may be `None` for empty positions in the grid.

    Attributes
    ----------
    cells : list of TableCell or None
        The cells in this group, with `None` for empty positions.
    bbox : BBox
        The bounding box of the entire group (x1, y1, x2, y2).
    """

    cells: list[TableCell | None]
    bbox: BBox

class Table:
    """
    Represents a table extracted from a PDF page.

    Attributes
    ----------
    bbox : BBox
        The bounding box of the entire table (x1, y1, x2, y2).
    cells : list of TableCell
        All cells contained in the table.
    rows : list of CellGroup
        All rows in the table, where each row contains cells or None for empty positions.
    columns : list of CellGroup
        All columns in the table, where each column contains cells or None for empty positions.
    page_index : int
        The index of the page containing this table.
    text_extracted : bool
        Whether text has been extracted for cells.
    """

    bbox: BBox
    cells: list[TableCell]
    rows: list[CellGroup]
    columns: list[CellGroup]
    page_index: int
    text_extracted: bool

    def to_csv(self) -> str:
        """
        Convert the table to a CSV formatted string.

        Returns
        -------
        str
            The table data formatted as a CSV string, with rows separated
            by newlines and cells separated by commas.

        Raises
        ------
        ValueError
            If text has not been extracted. Call extract_text first or
            use `extract_text=True` when finding tables.

        Examples
        --------
        >>> from tablers import Document, find_tables
        >>> doc = Document("example.pdf")
        >>> page = doc.get_page(0)
        >>> tables = find_tables(page, extract_text=True)
        >>> csv_content = tables[0].to_csv()
        >>> print(csv_content)
        """
        ...

    def to_markdown(self) -> str:
        """
        Convert the table to a Markdown formatted string.

        Returns
        -------
        str
            The table data formatted as a Markdown table string, with rows separated
            by newlines and cells separated by pipes. The first row is treated as
            the header row with a separator line below it.

        Raises
        ------
        ValueError
            If text has not been extracted. Call extract_text first or
            use `extract_text=True` when finding tables.

        Examples
        --------
        >>> from tablers import Document, find_tables
        >>> doc = Document("example.pdf")
        >>> page = doc.get_page(0)
        >>> tables = find_tables(page, extract_text=True)
        >>> markdown_content = tables[0].to_markdown()
        >>> print(markdown_content)
        | Header1 | Header2 |
        | --- | --- |
        | Cell1 | Cell2 |
        """
        ...

    def to_html(self) -> str:
        """
        Convert the table to an HTML formatted string.

        Returns
        -------
        str
            The table data formatted as an HTML table string, with rows wrapped
            in `<tr>` tags and cells wrapped in `<td>` tags. Special HTML
            characters are escaped.

        Raises
        ------
        ValueError
            If text has not been extracted. Call extract_text first or
            use `extract_text=True` when finding tables.

        Examples
        --------
        >>> from tablers import Document, find_tables
        >>> doc = Document("example.pdf")
        >>> page = doc.get_page(0)
        >>> tables = find_tables(page, extract_text=True)
        >>> html_content = tables[0].to_html()
        >>> print(html_content)
        <table>
        <tr><td>Header1</td><td>Header2</td></tr>
        <tr><td>Cell1</td><td>Cell2</td></tr>
        </table>
        """
        ...

class WordsExtractSettingsItems(TypedDict, total=False):
    """
    TypedDict for WordsExtractSettings keyword arguments.

    Attributes
    ----------
    x_tolerance : float
        X-axis tolerance for grouping characters into words. Default: 3.0
    y_tolerance : float
        Y-axis tolerance for grouping characters into lines. Default: 3.0
    keep_blank_chars : bool
        Whether to preserve blank/whitespace characters. Default: False
    use_text_flow : bool
        Whether to use the PDF's text flow order. Default: False
    text_read_in_clockwise : bool
        Whether text reads in clockwise direction. Default: True
    split_at_punctuation : {"all"} or str or None
        Punctuation splitting configuration. Default: None
    expand_ligatures : bool
        Whether to expand ligatures into individual characters. Default: True
    need_strip : bool
        Whether to strip leading/trailing whitespace from cell text. Default: True
    """

    x_tolerance: NonNegativeFloat  # Default: 3.0
    y_tolerance: NonNegativeFloat  # Default: 3.0
    keep_blank_chars: bool  # Default: False
    use_text_flow: bool  # Default: False
    text_read_in_clockwise: bool  # Default: True
    split_at_punctuation: Literal["all"] | str | None  # Default: None
    expand_ligatures: bool  # Default: True
    need_strip: bool  # Default: True

class WordsExtractSettings:
    """
    Settings for text/word extraction from PDF pages.

    Controls how characters are grouped into words, including
    tolerance values and text direction handling.

    Attributes
    ----------
    x_tolerance : float
        X-axis tolerance for grouping characters into words.
    y_tolerance : float
        Y-axis tolerance for grouping characters into lines.
    keep_blank_chars : bool
        Whether to preserve blank/whitespace characters.
    use_text_flow : bool
        Whether to use the PDF's text flow order.
    text_read_in_clockwise : bool
        Whether text reads in clockwise direction.
    split_at_punctuation : str or None
        Punctuation splitting configuration.
    expand_ligatures : bool
        Whether to expand ligatures into individual characters.
    need_strip : bool
        Whether to strip leading/trailing whitespace from cell text.

    Parameters
    ----------
    **kwargs : WordsExtractSettingsItems
        Optional keyword arguments to override default settings.
    """

    x_tolerance: float
    y_tolerance: float
    keep_blank_chars: bool
    use_text_flow: bool
    text_read_in_clockwise: bool
    split_at_punctuation: str | None
    expand_ligatures: bool
    need_strip: bool

    def __init__(self, **kwargs: Unpack[WordsExtractSettingsItems]) -> None: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...

class TfSettingItems(TypedDict, total=False):
    """
    TypedDict for TfSettings keyword arguments.

    Attributes
    ----------
    vertical_strategy : {"lines", "lines_strict", "text", "explicit"}
        Strategy for detecting vertical edges. Default: "lines_strict"
    horizontal_strategy : {"lines", "lines_strict", "text", "explicit"}
        Strategy for detecting horizontal edges. Default: "lines_strict"
    snap_x_tolerance : float
        Tolerance for snapping vertical edges together. Default: 3.0
    snap_y_tolerance : float
        Tolerance for snapping horizontal edges together. Default: 3.0
    join_x_tolerance : float
        Tolerance for joining horizontal edges. Default: 3.0
    join_y_tolerance : float
        Tolerance for joining vertical edges. Default: 3.0
    edge_min_length : float
        Minimum length for edges to be included. Default: 3.0
    edge_min_length_prefilter : float
        Minimum length for edges before merging. Default: 1.0
    min_words_vertical : int
        Minimum words for vertical text-based edge detection. Default: 3
    min_words_horizontal : int
        Minimum words for horizontal text-based edge detection. Default: 1
    intersection_x_tolerance : float
        X-tolerance for detecting edge intersections. Default: 3.0
    intersection_y_tolerance : float
        Y-tolerance for detecting edge intersections. Default: 3.0
    include_single_cell : bool
        Whether to include tables with only a single cell. Default: False
    min_rows : int or None
        Minimum number of rows required for a table. None means would not filter by this arg.
        Default: None
    min_columns : int or None
        Minimum number of columns required for a table. None means would not filter by this arg.
        Default: None
    text_x_tolerance : float
        X-tolerance for text extraction. Default: 3.0
    text_y_tolerance : float
        Y-tolerance for text extraction. Default: 3.0
    text_keep_blank_chars : bool
        Whether to keep blank characters in text. Default: False
    text_use_text_flow : bool
        Whether to use PDF text flow order. Default: False
    text_read_in_clockwise : bool
        Whether text reads clockwise. Default: True
    text_split_at_punctuation : {"all"} or str or None
        Punctuation splitting for text. Default: None
    text_expand_ligatures : bool
        Whether to expand ligatures in text. Default: True
    text_need_strip : bool
        Whether to strip leading/trailing whitespace from cell text. Default: True
    explicit_h_edges : list[Edge] or None
        Explicit horizontal edges to include in table detection. Default: None
    explicit_v_edges : list[Edge] or None
        Explicit vertical edges to include in table detection. Default: None
    """

    vertical_strategy: Literal[
        "lines", "lines_strict", "text", "explicit"
    ]  # Default: "lines_strict"
    horizontal_strategy: Literal[
        "lines", "lines_strict", "text", "explicit"
    ]  # Default: "lines_strict"
    snap_x_tolerance: NonNegativeFloat  # Default: 3.0
    snap_y_tolerance: NonNegativeFloat  # Default: 3.0
    join_x_tolerance: NonNegativeFloat  # Default: 3.0
    join_y_tolerance: NonNegativeFloat  # Default: 3.0
    edge_min_length: NonNegativeFloat  # Default: 3.0
    edge_min_length_prefilter: NonNegativeFloat  # Default: 1.0
    min_words_vertical: NonNegativeInt  # Default: 3
    min_words_horizontal: NonNegativeInt  # Default: 1
    intersection_x_tolerance: NonNegativeFloat  # Default: 3.0
    intersection_y_tolerance: NonNegativeFloat  # Default: 3.0
    include_single_cell: bool  # Default: False
    min_rows: int | None  # Default: None
    min_columns: int | None  # Default: None
    text_need_strip: bool  # Default: True
    text_x_tolerance: NonNegativeFloat  # Default: 3.0
    text_y_tolerance: NonNegativeFloat  # Default: 3.0
    text_keep_blank_chars: bool  # Default: False
    text_use_text_flow: bool  # Default: False
    text_read_in_clockwise: bool  # Default: True
    text_split_at_punctuation: Literal["all"] | str | None  # Default: None
    text_expand_ligatures: bool  # Default: True
    explicit_h_edges: list[Edge] | None  # Default: None
    explicit_v_edges: list[Edge] | None  # Default: None

class TfSettings:
    """
    Settings for table finding operations.

    Controls how edges are detected, snapped, joined, and how intersections
    are identified when finding tables in a PDF page.

    Attributes
    ----------
    vertical_strategy : {"lines", "lines_strict", "text", "explicit"}
        Strategy for detecting vertical edges.
    horizontal_strategy : {"lines", "lines_strict", "text", "explicit"}
        Strategy for detecting horizontal edges.
    snap_x_tolerance : float
        Tolerance for snapping vertical edges together.
    snap_y_tolerance : float
        Tolerance for snapping horizontal edges together.
    join_x_tolerance : float
        Tolerance for joining horizontal edges.
    join_y_tolerance : float
        Tolerance for joining vertical edges.
    edge_min_length : float
        Minimum length for edges to be included.
    edge_min_length_prefilter : float
        Minimum length for edges before merging.
    min_words_vertical : int
        Minimum words for vertical text-based edge detection.
    min_words_horizontal : int
        Minimum words for horizontal text-based edge detection.
    intersection_x_tolerance : float
        X-tolerance for detecting edge intersections.
    intersection_y_tolerance : float
        Y-tolerance for detecting edge intersections.
    include_single_cell : bool
        Whether to include tables with only a single cell.
    min_rows : int or None
        Minimum number of rows required for a table. None means would not filter by this arg.
    min_columns : int or None
        Minimum number of columns required for a table. None means would not filter by this arg.
    text_need_strip : bool
        Whether to strip leading/trailing whitespace from cell text.
    text_settings : WordsExtractSettings
        Settings for text/word extraction.
    text_x_tolerance : float
        X-tolerance for text extraction.
    text_y_tolerance : float
        Y-tolerance for text extraction.
    text_keep_blank_chars : bool
        Whether to keep blank characters in text.
    text_use_text_flow : bool
        Whether to use PDF text flow order.
    text_read_in_clockwise : bool
        Whether to read text in clockwise direction.
    text_split_at_punctuation : str or None
        Punctuation splitting for text.
    text_expand_ligatures : bool
        Whether to expand ligatures in text.
    explicit_h_edges : list[Edge] or None
        Explicit horizontal edges to include in table detection.
    explicit_v_edges : list[Edge] or None
        Explicit vertical edges to include in table detection.

    Parameters
    ----------
    **kwargs : TfSettingItems
        Optional keyword arguments to override default settings.
    """

    vertical_strategy: Literal["lines", "lines_strict", "text", "explicit"]
    horizontal_strategy: Literal["lines", "lines_strict", "text", "explicit"]
    snap_x_tolerance: float
    snap_y_tolerance: float
    join_x_tolerance: float
    join_y_tolerance: float
    edge_min_length: float
    edge_min_length_prefilter: float
    min_words_vertical: int
    min_words_horizontal: int
    intersection_x_tolerance: float
    intersection_y_tolerance: float
    include_single_cell: bool
    min_rows: int | None
    min_columns: int | None
    text_need_strip: bool
    text_settings: WordsExtractSettings
    text_x_tolerance: float
    text_y_tolerance: float
    text_keep_blank_chars: bool
    text_use_text_flow: bool
    text_read_in_clockwise: bool
    text_split_at_punctuation: str | None
    text_expand_ligatures: bool
    explicit_h_edges: list[Edge] | None
    explicit_v_edges: list[Edge] | None

    def __init__(self, **kwargs: Unpack[TfSettingItems]) -> None: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...

def find_all_cells_bboxes(
    page: Page | None = None,
    clip: BBox | None = None,
    tf_settings: TfSettings | None = None,
    **kwargs,
) -> list[BBox]:
    """
    Find all table cell bounding boxes in a PDF page or from explicit edges.

    Parameters
    ----------
    page : Page or None, optional
        The PDF page to analyze. Can be None only if both horizontal_strategy
        and vertical_strategy are set to "explicit".
    clip : BBox or None, optional
        Optional clip region (x1, y1, x2, y2). If provided, only edges within
        this region are used for cell detection. Edges intersecting the clip
        boundary are clipped to fit within it.
    tf_settings : TfSettings, optional
        TableFinder settings object. If not provided, default settings are used.
    **kwargs
        Additional keyword arguments passed to TfSettings.

    Returns
    -------
    list of BBox
        A list of bounding boxes (x1, y1, x2, y2) for each detected cell.

    Raises
    ------
    RuntimeError
        If page is None and either strategy is not "explicit".

    Examples
    --------
    >>> from tablers import Document, find_all_cells_bboxes
    >>> doc = Document("example.pdf")
    >>> page = doc.get_page(0)
    >>> cells = find_all_cells_bboxes(page)
    >>> print(f"Found {len(cells)} cells")

    Using explicit edges without a page:

    >>> from tablers import Edge, TfSettings, find_all_cells_bboxes
    >>> h_edges = [Edge("h", 0.0, 0.0, 100.0, 0.0), Edge("h", 0.0, 50.0, 100.0, 50.0)]
    >>> v_edges = [Edge("v", 0.0, 0.0, 0.0, 50.0), Edge("v", 100.0, 0.0, 100.0, 50.0)]
    >>> settings = TfSettings(
    ...     horizontal_strategy="explicit",
    ...     vertical_strategy="explicit",
    ...     explicit_h_edges=h_edges,
    ...     explicit_v_edges=v_edges,
    ... )
    >>> cells = find_all_cells_bboxes(None, tf_settings=settings)

    Using clip to extract cells from a specific region:

    >>> cells = find_all_cells_bboxes(page, clip=(100.0, 100.0, 400.0, 300.0))

    Warning
    -------
    When a page is marked as rotated by 90° or 270°, `page.width` and `page.height`
    are defined based on the upright orientation (as you would normally view the page).
    However, all object coordinates (lines, text, etc.) within the PDF are defined
    based on the unrotated coordinate system (where `page.width` corresponds to the
    actual `page.height` after rotation is removed).

    Therefore, `clip` values must also be specified using the unrotated coordinate
    system. Failing to account for this may result in incorrect cell detection.
    """
    ...

def find_tables_from_cells(
    cells: list[BBox],
    extract_text: bool,
    page: Page | None = None,
    tf_settings: TfSettings | None = None,
    **kwargs: Unpack[TfSettingItems],
) -> list[Table]:
    """
    Construct tables from a list of cell bounding boxes.

    Parameters
    ----------
    cells : list of BBox
        A list of cell bounding boxes to group into tables.
    extract_text : bool
        Whether to extract text content from cells.
    page : Page, optional
        The PDF page (required if extract_text is True).
    tf_settings : TfSettings, optional
        Table finder settings for text extraction and single cell filtering.
    **kwargs : TfSettingItems
        Additional keyword arguments for settings.

    Returns
    -------
    list of Table
        A list of Table objects constructed from the cells.

    Raises
    ------
    RuntimeError
        If extract_text is True but page is not provided.

    Examples
    --------
    >>> from tablers import Document, find_all_cells_bboxes, find_tables_from_cells
    >>> doc = Document("example.pdf")
    >>> page = doc.get_page(0)
    >>> cells = find_all_cells_bboxes(page)
    >>> tables = find_tables_from_cells(cells, extract_text=True, page=page)
    """
    ...

def find_tables(
    page: Page | None = None,
    extract_text: bool = True,
    clip: BBox | None = None,
    tf_settings: TfSettings | None = None,
    **kwargs: Unpack[TfSettingItems],
) -> list[Table]:
    """
    Find all tables in a PDF page or from explicit edges.

    This is the main entry point for table detection. It extracts edges,
    finds intersections, builds cells, and groups them into tables.

    Parameters
    ----------
    page : Page | None, optional
        The PDF page to analyze. Can be None only if both strategies are
        "explicit" and extract_text is False.
    extract_text : bool, default True
        Whether to extract text content from table cells.
    clip : BBox or None, optional
        Optional clip region (x1, y1, x2, y2). If provided, only edges within
        this region are used for table detection. Edges intersecting the clip
        boundary are clipped to fit within it.
    tf_settings : TfSettings, optional
        TableFinder settings object. If not provided, default settings are used.
    **kwargs : TfSettingItems
        Additional keyword arguments passed to TfSettings.

    Returns
    -------
    list of Table
        A list of Table objects found in the page.

    Raises
    ------
    ValueError
        If page is None and extract_text is True.
        If page is None and either strategy is not "explicit".

    Examples
    --------
    >>> from tablers import Document, find_tables
    >>> doc = Document("example.pdf")
    >>> page = doc.get_page(0)
    >>> tables = find_tables(page, extract_text=True)
    >>> for table in tables:
    ...     print(f"Table with {len(table.cells)} cells at {table.bbox}")

    Using explicit edges without a page:

    >>> from tablers import Edge, TfSettings, find_tables
    >>> h_edges = [Edge("h", 0.0, 0.0, 100.0, 0.0), Edge("h", 0.0, 100.0, 100.0, 100.0)]
    >>> v_edges = [Edge("v", 0.0, 0.0, 0.0, 100.0), Edge("v", 100.0, 0.0, 100.0, 100.0)]
    >>> settings = TfSettings(
    ...     horizontal_strategy="explicit",
    ...     vertical_strategy="explicit",
    ...     explicit_h_edges=h_edges,
    ...     explicit_v_edges=v_edges,
    ... )
    >>> tables = find_tables(page=None, extract_text=False, tf_settings=settings)

    Using clip to extract tables from a specific region:

    >>> tables = find_tables(page, clip=(100.0, 100.0, 400.0, 300.0))

    Warning
    -------
    When a page is marked as rotated by 90° or 270°, `page.width` and `page.height`
    are defined based on the upright orientation (as you would normally view the page).
    However, all object coordinates (lines, text, etc.) within the PDF are defined
    based on the unrotated coordinate system (where `page.width` corresponds to the
    actual `page.height` after rotation is removed).

    Therefore, `clip` values must also be specified using the unrotated coordinate
    system. Failing to account for this may result in incorrect table extraction.
    """
    ...

def get_edges(
    page: Page | None = None,
    tf_settings: TfSettings | None = None,
    **kwargs: Unpack[TfSettingItems],
) -> dict[Literal["v", "h"], list[Edge]]:
    """
    Extract edges (lines and rectangle borders) from a PDF page or from explicit edges.

    This function is primarily intended for debugging intermediate
    processing steps on the Python side.

    Parameters
    ----------
    page : Page or None, optional
        The PDF page to extract edges from. Can be None only if both
        horizontal_strategy and vertical_strategy are set to "explicit".
    tf_settings : TfSettings, optional
        TableFinder settings object. If not provided, default settings are used.
    **kwargs : TfSettingItems
        Additional keyword arguments passed to TfSettings.

    Returns
    -------
    dict
        A dictionary with keys "h" (horizontal edges) and "v" (vertical edges),
        each containing a list of Edge objects.

    Raises
    ------
    RuntimeError
        If page is None and either strategy is not "explicit".

    Examples
    --------
    >>> from tablers import Document, get_edges
    >>> doc = Document("example.pdf")
    >>> page = doc.get_page(0)
    >>> edges = get_edges(page)
    >>> print(f"Found {len(edges['h'])} horizontal and {len(edges['v'])} vertical edges")

    Using explicit edges without a page:

    >>> from tablers import Edge, get_edges
    >>> h_edge = Edge("h", 0.0, 50.0, 100.0, 50.0)
    >>> v_edge = Edge("v", 50.0, 0.0, 50.0, 100.0)
    >>> edges = get_edges(
    ...     None,
    ...     horizontal_strategy="explicit",
    ...     vertical_strategy="explicit",
    ...     explicit_h_edges=[h_edge],
    ...     explicit_v_edges=[v_edge],
    ... )
    """
    ...
