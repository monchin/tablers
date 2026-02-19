"""
Python Page wrapper around the Rust Pyo3Page binding.

This module provides a Page class that holds an inner Pyo3Page instance
and delegates all operations to it.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

from .tablers import Pyo3Page

if TYPE_CHECKING:
    from .__init__ import Document
    from .tablers import Objects


class Page:
    """
    Represents a single page in a PDF document.

    Holds an inner Pyo3Page instance and delegates all methods and properties
    to it. This allows the Python API to use a dedicated Page type while
    the Rust binding remains Pyo3Page.

    Attributes
    ----------
    inner : Pyo3Page
        The underlying Rust page binding.
    doc : Document
        The Python Document this page belongs to.
    """

    def __init__(self, inner: Pyo3Page, doc: Document) -> None:
        self.inner = inner
        self.doc = doc

    def __repr__(self) -> str:
        return (
            f"Page(idx={self.inner.page_idx}, width={self.inner.width}, height={self.inner.height})"
        )

    @property
    def width(self) -> float:
        return self.inner.width

    @property
    def height(self) -> float:
        return self.inner.height

    @property
    def page_idx(self) -> int:
        return self.inner.page_idx

    @property
    def rotation_degrees(self) -> float:
        return self.inner.rotation_degrees

    def is_valid(self) -> bool:
        return self.inner.is_valid()

    def extract_objects(self) -> None:
        self.inner.extract_objects()

    def clear_cache(self) -> None:
        """Clear the cached objects to free memory."""
        self.inner.clear_cache()

    def clear(self) -> None:
        """Alias for clear_cache(); kept for backward compatibility."""
        self.inner.clear_cache()

    @property
    def objects(self) -> Objects | None:
        return self.inner.objects
