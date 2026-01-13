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
"""

from __future__ import annotations

import platform
from pathlib import Path
from typing import Final

from .tablers import Document as RsDoc
from .tablers import (
    Edge,
    Page,
    PageIterator,
    PdfiumRuntime,
    TfSettings,
    WordsExtractSettings,
    __version__,
    find_all_cells_bboxes,
    find_tables,
    find_tables_from_cells,
    get_edges,
)
from .tablers import (
    find_all_tables as _find_all_tables,
)

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


__all__ = [
    "Document",
    "Edge",
    "Page",
    "PdfiumRuntime",
    "TfSettings",
    "WordsExtractSettings",
    "find_all_cells_bboxes",
    "find_all_tables",
    "find_tables_from_cells",
    "find_tables",
    "get_default_pdfium_path",
    "get_edges",
    "get_runtime",
    "__version__",
]


def find_all_tables(
    doc: Document,
    extract_text: bool = True,
    tf_settings: TfSettings | None = None,
    num_threads: int | None = None,
    batch_size: int | None = None,
    **kwargs,
) -> dict[int, list]:
    """
    Find all tables in all pages of a PDF document using multi-threading.

    This function extracts tables from all pages in parallel using a thread pool,
    bypassing Python's GIL limitation for CPU-intensive work.

    Parameters
    ----------
    doc : Document
        The PDF document to analyze.
    extract_text : bool, default True
        Whether to extract text content from table cells.
    tf_settings : TfSettings, optional
        TableFinder settings object. If not provided, default settings are used.
    num_threads : int or None, optional
        Number of threads to use for parallel processing. If None or not specified,
        uses the number of CPU cores available.
    batch_size : int or None, optional
        Number of pages to process per batch. Using batches can reduce memory usage
        for large documents. If None or not specified, all pages are processed at once.
    **kwargs
        Additional keyword arguments passed to TfSettings.

    Returns
    -------
    dict[int, list[Table]]
        A dictionary mapping page indices (0-based) to lists of Table objects
        found on each page. Pages with no tables are not included in the result.

    Examples
    --------
    >>> from tablers import Document, find_all_tables
    >>> with Document("example.pdf") as doc:
    ...     all_tables = find_all_tables(doc, extract_text=True)
    ...     for page_idx, tables in all_tables.items():
    ...         print(f"Page {page_idx}: {len(tables)} tables")

    Using batch processing for large documents:

    >>> all_tables = find_all_tables(doc, batch_size=50)
    """
    return _find_all_tables(
        doc.doc,
        extract_text=extract_text,
        tf_settings=tf_settings,
        num_threads=num_threads,
        batch_size=batch_size,
        **kwargs,
    )


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
    """

    def __init__(
        self,
        path: Path | str | None = None,
        bytes: bytes | None = None,
        password: str | None = None,
    ):
        self.doc = RsDoc(
            PDFIUM_RT,
            path=str(path) if path is not None else None,
            bytes=bytes,
            password=password,
        )

    def __enter__(self):
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
        return self.doc.get_page(page_num)

    def pages(self) -> PageIterator:
        """
        Get an iterator over all pages in the document.

        This method is memory-efficient for large PDFs as it loads
        pages on demand rather than all at once.

        Returns
        -------
        PageIterator
            An iterator that yields pages one at a time.

        Examples
        --------
        >>> doc = Document("example.pdf")
        >>> for page in doc.pages():
        ...     print(f"Page size: {page.width} x {page.height}")
        """
        return self.doc.pages()

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
