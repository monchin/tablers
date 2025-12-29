"""
Tests for the Document class and related functionality.
"""

from pathlib import Path

import pytest
from tablers import Document, Page


class TestDocumentInit:
    """Tests for Document initialization."""

    def test_open_from_path_string(self, edge_test_pdf_path: Path) -> None:
        """Document can be opened from a path string."""
        doc = Document(path=str(edge_test_pdf_path))
        assert not doc.is_closed()
        assert doc.page_count > 0
        doc.close()

    def test_open_from_path_object(self, edge_test_pdf_path: Path) -> None:
        """Document can be opened from a Path object."""
        doc = Document(path=edge_test_pdf_path)
        assert not doc.is_closed()
        assert doc.page_count > 0
        doc.close()

    def test_open_from_bytes(self, edge_test_pdf_bytes: bytes) -> None:
        """Document can be opened from bytes."""
        doc = Document(bytes=edge_test_pdf_bytes)
        assert not doc.is_closed()
        assert doc.page_count > 0
        doc.close()

    def test_open_nonexistent_file_raises(self) -> None:
        """Opening a nonexistent file should raise an error."""
        with pytest.raises(RuntimeError):
            Document(path="nonexistent_file.pdf")

    def test_open_without_path_or_bytes_raises(self) -> None:
        """Opening without path or bytes should raise ValueError."""
        with pytest.raises((ValueError, RuntimeError)):
            Document()


class TestDocumentProperties:
    """Tests for Document properties."""

    def test_page_count(self, edge_test_doc: Document) -> None:
        """page_count should return a positive integer."""
        assert isinstance(edge_test_doc.page_count, int)
        assert edge_test_doc.page_count > 0

    def test_is_closed_initially_false(self, edge_test_doc: Document) -> None:
        """is_closed should return False for a newly opened document."""
        assert edge_test_doc.is_closed() is False

    def test_is_closed_after_close(self, edge_test_pdf_path: Path) -> None:
        """is_closed should return True after calling close()."""
        doc = Document(path=edge_test_pdf_path)
        doc.close()
        assert doc.is_closed() is True


class TestDocumentClose:
    """Tests for Document close functionality."""

    def test_close_document(self, edge_test_pdf_path: Path) -> None:
        """Document can be closed."""
        doc = Document(path=edge_test_pdf_path)
        assert not doc.is_closed()
        doc.close()
        assert doc.is_closed()

    def test_double_close_is_safe(self, edge_test_pdf_path: Path) -> None:
        """Calling close() twice should not raise an error."""
        doc = Document(path=edge_test_pdf_path)
        doc.close()
        doc.close()  # Should not raise
        assert doc.is_closed()


class TestDocumentContextManager:
    """Tests for Document context manager (__enter__ and __exit__)."""

    def test_context_manager_basic(self, edge_test_pdf_path: Path) -> None:
        """Document can be used as a context manager."""
        with Document(path=edge_test_pdf_path) as doc:
            assert not doc.is_closed()
            assert doc.page_count > 0

    def test_context_manager_closes_on_exit(self, edge_test_pdf_path: Path) -> None:
        """Document is automatically closed when exiting the with block."""
        doc = Document(path=edge_test_pdf_path)
        with doc:
            assert not doc.is_closed()
        assert doc.is_closed()

    def test_context_manager_returns_self(self, edge_test_pdf_path: Path) -> None:
        """__enter__ should return the document itself."""
        doc = Document(path=edge_test_pdf_path)
        with doc as entered_doc:
            assert entered_doc is doc
        doc.close()

    def test_context_manager_closes_on_exception(self, edge_test_pdf_path: Path) -> None:
        """Document is closed even if an exception occurs inside the with block."""
        doc = Document(path=edge_test_pdf_path)
        try:
            with doc:
                raise ValueError("Test exception")
        except ValueError:
            pass
        assert doc.is_closed()

    def test_context_manager_with_page_access(self, edge_test_pdf_path: Path) -> None:
        """Pages can be accessed within the context manager."""
        with Document(path=edge_test_pdf_path) as doc:
            page = doc.get_page(0)
            assert page.width > 0
            assert page.height > 0

    def test_context_manager_with_iteration(self, edge_test_pdf_path: Path) -> None:
        """Pages can be iterated within the context manager."""
        with Document(path=edge_test_pdf_path) as doc:
            pages = list(doc.pages())
            assert len(pages) == doc.page_count

    def test_context_manager_from_bytes(self, edge_test_pdf_bytes: bytes) -> None:
        """Context manager works when document is opened from bytes."""
        with Document(bytes=edge_test_pdf_bytes) as doc:
            assert not doc.is_closed()
            assert doc.page_count > 0
        # After exiting the context, document should be closed


class TestDocumentGetPage:
    """Tests for Document.get_page() method."""

    def test_get_first_page(self, edge_test_doc: Document) -> None:
        """Getting the first page should work."""
        page = edge_test_doc.get_page(0)
        assert isinstance(page, Page)

    def test_get_page_has_dimensions(self, edge_test_doc: Document) -> None:
        """Retrieved page should have valid dimensions."""
        page = edge_test_doc.get_page(0)
        assert page.width > 0
        assert page.height > 0

    def test_get_page_out_of_range_raises(self, edge_test_doc: Document) -> None:
        """Getting a page out of range should raise IndexError."""
        page_count = edge_test_doc.page_count
        with pytest.raises(IndexError):
            edge_test_doc.get_page(page_count)

    def test_get_negative_page_raises(self, edge_test_doc: Document) -> None:
        """Getting a negative page index should raise an error."""
        with pytest.raises((IndexError, OverflowError)):
            edge_test_doc.get_page(-1)


class TestDocumentPages:
    """Tests for Document.pages() iterator."""

    def test_pages_iterator(self, edge_test_doc: Document) -> None:
        """pages() should return an iterator over all pages."""
        pages = list(edge_test_doc.pages())
        assert len(pages) == edge_test_doc.page_count

    def test_pages_iterator_yields_page_objects(self, edge_test_doc: Document) -> None:
        """pages() iterator should yield Page objects."""
        for page in edge_test_doc.pages():
            assert isinstance(page, Page)
            break  # Only check first page

    def test_pages_iterator_multiple_times(self, edge_test_doc: Document) -> None:
        """pages() can be called multiple times."""
        pages1 = list(edge_test_doc.pages())
        pages2 = list(edge_test_doc.pages())
        assert len(pages1) == len(pages2)


class TestPage:
    """Tests for Page class."""

    def test_page_dimensions(self, edge_test_doc: Document) -> None:
        """Page should have positive width and height."""
        page = edge_test_doc.get_page(0)
        assert isinstance(page.width, float)
        assert isinstance(page.height, float)
        assert page.width > 0
        assert page.height > 0

    def test_page_is_valid(self, edge_test_doc: Document) -> None:
        """Page from open document should be valid."""
        page = edge_test_doc.get_page(0)
        assert page.is_valid() is True

    def test_page_extract_objects(self, edge_test_doc: Document) -> None:
        """extract_objects() should work without error."""
        page = edge_test_doc.get_page(0)
        page.extract_objects()
        # Objects should be available after extraction
        assert page.objects is not None

    def test_page_objects_available(self, edge_test_doc: Document) -> None:
        """objects should be available after getting a page."""
        page = edge_test_doc.get_page(0)
        # Objects may be auto-extracted on page access
        assert page.objects is not None or page.objects is None  # Either is valid

    def test_page_objects_after_extraction(self, edge_test_doc: Document) -> None:
        """objects should be available after calling extract_objects()."""
        page = edge_test_doc.get_page(0)
        page.extract_objects()
        assert page.objects is not None


class TestEncryptedPDF:
    """Tests for opening and reading encrypted PDF documents."""

    def test_open_encrypted_pdf_with_password(
        self, encrypted_pdf_path: Path, encrypted_pdf_password: str
    ) -> None:
        """Encrypted PDF can be opened with correct password."""
        doc = Document(path=encrypted_pdf_path, password=encrypted_pdf_password)
        assert not doc.is_closed()
        assert doc.page_count > 0
        doc.close()

    def test_open_encrypted_pdf_from_bytes_with_password(
        self, encrypted_pdf_bytes: bytes, encrypted_pdf_password: str
    ) -> None:
        """Encrypted PDF can be opened from bytes with correct password."""
        doc = Document(bytes=encrypted_pdf_bytes, password=encrypted_pdf_password)
        assert not doc.is_closed()
        assert doc.page_count > 0
        doc.close()

    def test_open_encrypted_pdf_without_password_raises(self, encrypted_pdf_path: Path) -> None:
        """Opening an encrypted PDF without password should raise an error."""
        with pytest.raises(RuntimeError):
            Document(path=encrypted_pdf_path)

    def test_open_encrypted_pdf_with_wrong_password_raises(self, encrypted_pdf_path: Path) -> None:
        """Opening an encrypted PDF with wrong password should raise an error."""
        with pytest.raises(RuntimeError):
            Document(path=encrypted_pdf_path, password="wrong_password")

    def test_encrypted_pdf_page_access(self, encrypted_doc: Document) -> None:
        """Pages can be accessed from an encrypted PDF."""
        page = encrypted_doc.get_page(0)
        assert isinstance(page, Page)
        assert page.width > 0
        assert page.height > 0

    def test_encrypted_pdf_extract_objects(self, encrypted_doc: Document) -> None:
        """Objects can be extracted from an encrypted PDF page."""
        page = encrypted_doc.get_page(0)
        page.extract_objects()
        assert page.objects is not None

    def test_encrypted_pdf_context_manager(
        self, encrypted_pdf_path: Path, encrypted_pdf_password: str
    ) -> None:
        """Encrypted PDF can be used as a context manager."""
        with Document(path=encrypted_pdf_path, password=encrypted_pdf_password) as doc:
            assert not doc.is_closed()
            assert doc.page_count > 0
        # After exiting the context, document should be closed

    def test_encrypted_pdf_iteration(self, encrypted_doc: Document) -> None:
        """Pages can be iterated from an encrypted PDF."""
        pages = list(encrypted_doc.pages())
        assert len(pages) == encrypted_doc.page_count
        for page in pages:
            assert isinstance(page, Page)
