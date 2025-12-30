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
    orietation : {"h", "v"}
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

    orietation: Literal["h", "v"]
    x1: float
    y1: float
    x2: float
    y2: float
    width: float
    color: Color

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

class Table:
    """
    Represents a table extracted from a PDF page.

    Attributes
    ----------
    bbox : BBox
        The bounding box of the entire table (x1, y1, x2, y2).
    cells : list of TableCell
        All cells contained in the table.
    """

    bbox: BBox
    cells: list[TableCell]

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
    """

    x_tolerance: NonNegativeFloat  # Default: 3.0
    y_tolerance: NonNegativeFloat  # Default: 3.0
    keep_blank_chars: bool  # Default: False
    use_text_flow: bool  # Default: False
    text_read_in_clockwise: bool  # Default: True
    split_at_punctuation: Literal["all"] | str | None  # Default: None
    expand_ligatures: bool  # Default: True

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

    def __init__(self, **kwargs: Unpack[WordsExtractSettingsItems]) -> None: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...

class TfSettingItems(TypedDict, total=False):
    """
    TypedDict for TfSettings keyword arguments.

    Attributes
    ----------
    vertical_strategy : {"lines", "lines_strict", "text"}
        Strategy for detecting vertical edges. Default: "lines_strict"
    horizontal_strategy : {"lines", "lines_strict", "text"}
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
    """

    vertical_strategy: Literal["lines", "lines_strict", "text"]  # Default: "lines_strict"
    horizontal_strategy: Literal["lines", "lines_strict", "text"]  # Default: "lines_strict"
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
    text_x_tolerance: NonNegativeFloat  # Default: 3.0
    text_y_tolerance: NonNegativeFloat  # Default: 3.0
    text_keep_blank_chars: bool  # Default: False
    text_use_text_flow: bool  # Default: False
    text_read_in_clockwise: bool  # Default: True
    text_split_at_punctuation: Literal["all"] | str | None  # Default: None
    text_expand_ligatures: bool  # Default: True

class TfSettings:
    """
    Settings for table finding operations.

    Controls how edges are detected, snapped, joined, and how intersections
    are identified when finding tables in a PDF page.

    Attributes
    ----------
    vertical_strategy : {"lines", "lines_strict", "text"}
        Strategy for detecting vertical edges.
    horizontal_strategy : {"lines", "lines_strict", "text"}
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

    Parameters
    ----------
    **kwargs : TfSettingItems
        Optional keyword arguments to override default settings.
    """

    vertical_strategy: Literal["lines", "lines_strict", "text"]
    horizontal_strategy: Literal["lines", "lines_strict", "text"]
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
    text_settings: WordsExtractSettings
    text_x_tolerance: float
    text_y_tolerance: float
    text_keep_blank_chars: bool
    text_use_text_flow: bool
    text_read_in_clockwise: bool
    text_split_at_punctuation: str | None
    text_expand_ligatures: bool

    def __init__(self, **kwargs: Unpack[TfSettingItems]) -> None: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...

def find_all_cells_bboxes(
    page: Page, tf_settings: TfSettings | None = None, **kwargs
) -> list[BBox]:
    """
    Find all table cell bounding boxes in a PDF page.

    Parameters
    ----------
    page : Page
        The PDF page to analyze.
    tf_settings : TfSettings, optional
        TableFinder settings object. If not provided, default settings are used.
    **kwargs
        Additional keyword arguments passed to TfSettings.

    Returns
    -------
    list of BBox
        A list of bounding boxes (x1, y1, x2, y2) for each detected cell.

    Examples
    --------
    >>> from tablers import Document, find_all_cells_bboxes
    >>> doc = Document("example.pdf")
    >>> page = doc.get_page(0)
    >>> cells = find_all_cells_bboxes(page)
    >>> print(f"Found {len(cells)} cells")
    """
    ...

def find_tables_from_cells(
    cells: list[BBox],
    extract_text: bool,
    page: Page | None = None,
    we_settings: WordsExtractSettings | None = None,
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
    we_settings : WordsExtractSettings, optional
        Word extraction settings for text extraction.
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
    page: Page,
    extract_text: bool,
    tf_settings: TfSettings | None = None,
    **kwargs: Unpack[TfSettingItems],
) -> list[Table]:
    """
    Find all tables in a PDF page.

    This is the main entry point for table detection. It extracts edges,
    finds intersections, builds cells, and groups them into tables.

    Parameters
    ----------
    page : Page
        The PDF page to analyze.
    extract_text : bool
        Whether to extract text content from table cells.
    tf_settings : TfSettings, optional
        TableFinder settings object. If not provided, default settings are used.
    **kwargs : TfSettingItems
        Additional keyword arguments passed to TfSettings.

    Returns
    -------
    list of Table
        A list of Table objects found in the page.

    Examples
    --------
    >>> from tablers import Document, find_tables
    >>> doc = Document("example.pdf")
    >>> page = doc.get_page(0)
    >>> tables = find_tables(page, extract_text=True)
    >>> for table in tables:
    ...     print(f"Table with {len(table.cells)} cells at {table.bbox}")
    """
    ...

def get_edges(page: Page, settings: TfSettingItems | None = None) -> dict[str, list[Edge]]:
    """
    Extract edges (lines and rectangle borders) from a PDF page.

    Parameters
    ----------
    page : Page
        The PDF page to extract edges from.
    settings : TfSettingItems, optional
        Dictionary of settings for edge extraction.

    Returns
    -------
    dict
        A dictionary with keys "h" (horizontal edges) and "v" (vertical edges),
        each containing a list of Edge objects.

    Examples
    --------
    >>> from tablers import Document, get_edges
    >>> doc = Document("example.pdf")
    >>> page = doc.get_page(0)
    >>> edges = get_edges(page)
    >>> print(f"Found {len(edges['h'])} horizontal and {len(edges['v'])} vertical edges")
    """
    ...
