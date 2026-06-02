"""
Tablers: A fast PDF table extraction library.

This module provides tools for extracting tables from PDF documents
using edge detection and cell identification algorithms.

The library is implemented in Rust for performance and exposed to
Python via PyO3 bindings.

Examples
--------
Basic usage for extracting tables from a PDF:

>>> from tablers import Document, find_tables
>>> doc = Document("example.pdf")
>>> for page in doc.pages():
...     tables = find_tables(page, extract_text=True)
...     for table in tables:
...         print(f"Found table with {len(table.cells)} cells")
>>> doc.close()

Notes
-----
The library automatically loads the appropriate Pdfium library
based on the operating system (Windows, Linux, or macOS).

**Thread Safety**: This library is NOT thread-safe. A global
``PDFIUM_RT`` is created at import time and is bound to the
importing thread. All ``Document`` operations must run on that
same thread. Using ``Document`` from a different thread will
raise a ``PanicException``. In multi-threaded environments,
import and use this library within the same worker thread, or
use ``multiprocessing`` instead of ``threading``.

Although `pdfium-render <https://github.com/ajrcarey/pdfium-render#multi-threading>`_
offers a ``thread_safe`` compile-time feature (mutex-based locking),
enabling it would introduce performance overhead in single-threaded
use, so tablers does not enable it.

**Pickle Support**: All pure-data objects (``Table``, ``TableCell``,
``Edge``, ``TfSettings``, ``WordsExtractSettings``, ``Objects``,
``Rect``, ``Line``, ``Char``, ``CellGroup``,
``TableCellValue``) support the Python pickle protocol and can be
passed between processes via ``multiprocessing``. The Pdfium-bound
types (``PdfiumRuntime``, ``Document``, ``Pyo3Page``) are not
picklable.
"""

from __future__ import annotations

import platform
from collections.abc import Iterator
from pathlib import Path
from typing import Final

from .page import Page
from .tablers import (
    Edge,
    PdfiumRuntime,
    Pyo3Doc,
    Pyo3Page,
    TfSettings,
    WordsExtractSettings,
    __version__,
    get_intersections_from_edges,
)
from .tablers import find_all_cells_bboxes as _find_all_cells_bboxes
from .tablers import find_tables as _find_tables
from .tablers import find_tables_from_cells as _find_tables_from_cells
from .tablers import get_edges as _get_edges
from .typing import BBox, Color, NonNegativeFloat, NonNegativeInt, Point

SYSTEM: Final = platform.system()

# Default pdfium library paths based on the operating system
PKG_DIR: Final = Path(__file__).parent
_PDFIUM_PATHS: Final = {
    "Windows": PKG_DIR / "pdfium.dll",
    "Linux": PKG_DIR / "libpdfium.so.1",
    "Darwin": PKG_DIR / "libpdfium.dylib",
}


def get_default_pdfium_path() -> Path:
    """
    Get the default path to the bundled Pdfium library for the current OS.

    Returns
    -------
    Path
        The path to the bundled Pdfium dynamic library.

    Raises
    ------
    RuntimeError
        If the current operating system is not supported.
    """
    if SYSTEM not in _PDFIUM_PATHS:
        raise RuntimeError(f"Unsupported system: {SYSTEM}")
    return _PDFIUM_PATHS[SYSTEM]


def get_runtime(path: Path | str | None = None) -> PdfiumRuntime:
    """
    Get a PdfiumRuntime instance, reusing the existing one if already initialized.

    If the Pdfium library has already been initialized (either from Python or Rust),
    the existing instance is reused and the provided path is ignored.

    Parameters
    ----------
    path : Path or str, optional
        The path to the Pdfium dynamic library.
        If not provided, the bundled library path is used.

    Returns
    -------
    PdfiumRuntime
        A PdfiumRuntime instance.

    Examples
    --------
    >>> runtime = get_runtime()  # Uses bundled library
    >>> runtime = get_runtime("/custom/path/to/pdfium.dll")  # Custom path (only used on first call)
    """
    if path is None:
        path = get_default_pdfium_path()
    return PdfiumRuntime(str(path))


# Initialize the global runtime using the default path
# This will reuse an existing instance if already initialized from Rust
PDFIUM_RT = get_runtime()


def _unwrap_page(page: Page | Pyo3Page | None) -> Pyo3Page | None:
    """Pass through to Rust: use inner when page is our Page wrapper."""
    if page is None:
        return None
    return page.inner if isinstance(page, Page) else page


def find_tables(
    page: Page | Pyo3Page | None = None,
    extract_text: bool = True,
    clip=None,
    tf_settings=None,
    **kwargs,
):
    """
    Find all tables in a PDF page.

    Thin wrapper around the Rust ``find_tables`` binding that accepts either
    a :class:`Page` wrapper or a raw :class:`Pyo3Page` for the *page* argument.
    All other parameters are forwarded unchanged; see the Rust binding's
    docstring (:func:`tablers.tablers.find_tables`) for the full parameter list.

    Parameters
    ----------
    page : Page or Pyo3Page, optional
        The page to extract tables from.
    extract_text : bool
        Whether to extract text content from cells.
    clip : BBox, optional
        Clip region ``(x0, y0, x1, y1)`` in page coordinates.
    tf_settings : TfSettings, optional
        Table-finder settings; keyword arguments are accepted as an alternative.
    **kwargs
        Additional ``TfSettings`` fields passed as keyword arguments.

    Returns
    -------
    list[Table]
        Detected tables with their cells and optional text content.
    """
    return _find_tables(
        page=_unwrap_page(page),
        extract_text=extract_text,
        clip=clip,
        tf_settings=tf_settings,
        **kwargs,
    )


def find_all_cells_bboxes(
    page: Page | Pyo3Page | None = None,
    clip=None,
    tf_settings=None,
    **kwargs,
):
    """
    Find all table cell bounding boxes in a PDF page.

    Thin wrapper around the Rust ``find_all_cells_bboxes`` binding that accepts
    either a :class:`Page` wrapper or a raw :class:`Pyo3Page` for *page*.
    All other parameters are forwarded unchanged.

    Parameters
    ----------
    page : Page or Pyo3Page, optional
        The page to extract cell bounding boxes from.
    clip : BBox, optional
        Clip region ``(x0, y0, x1, y1)`` in page coordinates.
    tf_settings : TfSettings, optional
        Table-finder settings; keyword arguments are accepted as an alternative.
    **kwargs
        Additional ``TfSettings`` fields passed as keyword arguments.

    Returns
    -------
    list[BBox]
        Bounding boxes of all detected table cells.
    """
    return _find_all_cells_bboxes(
        page=_unwrap_page(page),
        clip=clip,
        tf_settings=tf_settings,
        **kwargs,
    )


def find_tables_from_cells(
    cells: list[BBox],
    extract_text: bool,
    page: Page | Pyo3Page | None = None,
    tf_settings=None,
    **kwargs,
):
    """
    Build tables from pre-computed cell bounding boxes.

    Thin wrapper around the Rust ``find_tables_from_cells`` binding that accepts
    either a :class:`Page` wrapper or a raw :class:`Pyo3Page` for *page*.

    Parameters
    ----------
    cells : list[BBox]
        Cell bounding boxes as returned by :func:`find_all_cells_bboxes`.
    extract_text : bool
        Whether to extract text content from cells.  Requires *page* when
        ``True``.
    page : Page or Pyo3Page, optional
        The source page, required when *extract_text* is ``True``.
    tf_settings : TfSettings, optional
        Table-finder settings; keyword arguments are accepted as an alternative.
    **kwargs
        Additional ``TfSettings`` fields passed as keyword arguments.

    Returns
    -------
    list[Table]
        Tables constructed from the provided cell bounding boxes.

    Notes
    -----
    The parameter was renamed from ``pdf_page`` to ``page`` in version 0.5.
    Passing ``pdf_page`` still works but raises a :class:`DeprecationWarning`.
    """
    import warnings

    if "pdf_page" in kwargs:
        warnings.warn(
            (
                "The 'pdf_page' parameter is deprecated and will be removed in a future version. "
                "Use 'page' instead."
            ),
            DeprecationWarning,
            stacklevel=2,
        )
        if page is None:
            page = kwargs.pop("pdf_page")
        else:
            kwargs.pop("pdf_page")

    return _find_tables_from_cells(
        cells=cells,
        extract_text=extract_text,
        page=_unwrap_page(page),
        tf_settings=tf_settings,
        **kwargs,
    )


def get_edges(
    page: Page | Pyo3Page | None = None,
    tf_settings=None,
    **kwargs,
):
    """
    Extract edges from a PDF page.

    Thin wrapper around the Rust ``get_edges`` binding that accepts either a
    :class:`Page` wrapper or a raw :class:`Pyo3Page` for *page*.

    Parameters
    ----------
    page : Page or Pyo3Page, optional
        The page to extract edges from.
    tf_settings : TfSettings, optional
        Table-finder settings; keyword arguments are accepted as an alternative.
    **kwargs
        Additional ``TfSettings`` fields passed as keyword arguments.

    Returns
    -------
    dict[str, list[Edge]]
        Detected edges grouped by direction (``"h"`` / ``"v"``).
    """
    return _get_edges(
        page=_unwrap_page(page),
        tf_settings=tf_settings,
        **kwargs,
    )


__all__ = [
    "BBox",
    "Color",
    "Document",
    "Edge",
    "NonNegativeFloat",
    "NonNegativeInt",
    "Page",
    "PdfiumRuntime",
    "Point",
    "TfSettings",
    "WordsExtractSettings",
    "find_all_cells_bboxes",
    "find_tables_from_cells",
    "find_tables",
    "get_default_pdfium_path",
    "get_edges",
    "get_intersections_from_edges",
    "get_runtime",
    "__version__",
]


class Document:
    """
    Represents an opened PDF document.

    Provides a high-level interface for working with PDF documents,
    including page access and iteration.

    Parameters
    ----------
    path : Path or str, optional
        File path to the PDF document.
    bytes : bytes, optional
        PDF content as bytes.
    password : str, optional
        Password for encrypted PDFs.

    Raises
    ------
    RuntimeError
        If the PDF cannot be opened or parsed.
    ValueError
        If neither path nor bytes is provided.

    Examples
    --------
    Open a PDF from a file path:

    >>> doc = Document("example.pdf")
    >>> print(f"Document has {doc.page_count} pages")
    >>> doc.close()

    Open a PDF from bytes:

    >>> with open("example.pdf", "rb") as f:
    ...     pdf_bytes = f.read()
    >>> doc = Document(bytes=pdf_bytes)
    >>> doc.close()

    Notes
    -----
    Either `path` or `bytes` must be provided, but not both.
    Always close the document when done to release resources.

    **Thread Safety**: This class is NOT thread-safe. All
    operations must be performed on the same thread that
    imported the ``tablers`` module. Using it from a different
    thread will raise a ``PanicException``.
    """

    _stream: bytes | None  # type hint only; instance value set in __init__

    def __init__(
        self,
        path: Path | str | None = None,
        bytes: bytes | None = None,
        password: str | None = None,
    ):
        self.doc = Pyo3Doc(
            PDFIUM_RT,
            path=str(path) if path is not None else None,
            bytes=bytes,
            password=password,
        )
        self._stream = None

    def __enter__(self) -> Document:
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        self.close()

    @property
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
        return self.doc.page_count()

    def save_to_bytes(self) -> bytes:
        """
        Serialize the document to bytes, **always without encryption**.

        Internally creates a new, empty PDF, copies every page from the current
        document into it, and serializes the result via ``FPDF_SaveAsCopy``.
        The returned bytes can always be opened without a password—even when the
        source was an encrypted PDF that was unlocked with a password.

        .. warning::
            If the original document was password-protected, this method
            **strips the encryption**.  Ensure this is intentional before
            distributing or persisting the result.

        .. note::
            This method is **not** cheap.  It allocates a full in-memory copy
            of the PDF on every call.  Cache the result if you need it more
            than once; do not call it in a loop.

        Returns
        -------
        bytes
            The serialized PDF content.

        Raises
        ------
        RuntimeError
            If the document has been closed or serialization fails.
        """
        if self.doc.is_closed():
            raise RuntimeError("Cannot serialize document: document has been closed")
        if self._stream is None:
            self._stream = self.doc.save_to_bytes()
        return self._stream

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
            The requested page object.

        Raises
        ------
        IndexError
            If the page index is out of range.
        RuntimeError
            If the document has been closed.

        Examples
        --------
        >>> doc = Document("example.pdf")
        >>> first_page = doc.get_page(0)
        >>> print(f"Page size: {first_page.width} x {first_page.height}")
        """
        return Page(self.doc.get_page(page_num), self)

    def pages(self) -> Iterator[Page]:
        """
        Get an iterator over all pages in the document.

        This method is memory-efficient for large PDFs as it loads
        pages on demand rather than all at once.

        Returns
        -------
        Iterator[Page]
            An iterator that yields pages one at a time.

        Examples
        --------
        >>> doc = Document("example.pdf")
        >>> for page in doc.pages():
        ...     print(f"Page size: {page.width} x {page.height}")
        """
        return (Page(p, self) for p in self.doc.pages())

    def close(self) -> None:
        """
        Close the document and release resources.

        After calling this method, all Page objects from this document
        become invalid and should not be used.

        Examples
        --------
        >>> doc = Document("example.pdf")
        >>> # ... work with document ...
        >>> doc.close()
        >>> doc.is_closed()
        True
        """
        self.doc.close()
        self._stream = None

    def is_closed(self) -> bool:
        """
        Check if the document has been closed.

        Returns
        -------
        bool
            True if the document is closed, False otherwise.

        Examples
        --------
        >>> doc = Document("example.pdf")
        >>> doc.is_closed()
        False
        >>> doc.close()
        >>> doc.is_closed()
        True
        """
        return self.doc.is_closed()
