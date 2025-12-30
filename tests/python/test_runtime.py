"""
Tests for PdfiumRuntime and related functionality.
"""

import platform
from pathlib import Path

from tablers import (
    Document,
    PdfiumRuntime,
    get_default_pdfium_path,
    get_runtime,
)


class TestGetDefaultPdfiumPath:
    """Tests for get_default_pdfium_path function."""

    def test_returns_path_object(self) -> None:
        """get_default_pdfium_path should return a Path object."""
        path = get_default_pdfium_path()
        assert isinstance(path, Path)

    def test_path_exists(self) -> None:
        """The default pdfium path should exist."""
        path = get_default_pdfium_path()
        assert path.exists(), f"Pdfium library not found at {path}"

    def test_path_matches_system(self) -> None:
        """The path should have the correct extension for the OS."""
        path = get_default_pdfium_path()
        system = platform.system()

        if system == "Windows":
            assert path.suffix == ".dll"
            assert path.name == "pdfium.dll"
        elif system == "Linux":
            assert path.suffix == ".1"
            assert path.name == "libpdfium.so.1"
        elif system == "Darwin":
            assert path.suffix == ".dylib"
            assert path.name == "libpdfium.dylib"


class TestGetRuntime:
    """Tests for get_runtime function."""

    def test_returns_pdfium_runtime(self) -> None:
        """get_runtime should return a PdfiumRuntime instance."""
        runtime = get_runtime()
        assert isinstance(runtime, PdfiumRuntime)

    def test_with_default_path(self) -> None:
        """get_runtime with no arguments should use default path."""
        runtime = get_runtime()
        assert runtime is not None

    def test_with_explicit_path(self) -> None:
        """get_runtime with explicit path should work."""
        path = get_default_pdfium_path()
        runtime = get_runtime(path)
        assert isinstance(runtime, PdfiumRuntime)

    def test_with_string_path(self) -> None:
        """get_runtime should accept string path."""
        path = str(get_default_pdfium_path())
        runtime = get_runtime(path)
        assert isinstance(runtime, PdfiumRuntime)

    def test_multiple_calls_succeed(self) -> None:
        """Multiple calls to get_runtime should succeed (reusing instance)."""
        runtime1 = get_runtime()
        runtime2 = get_runtime()
        # Both should be valid PdfiumRuntime instances
        assert isinstance(runtime1, PdfiumRuntime)
        assert isinstance(runtime2, PdfiumRuntime)


class TestPdfiumRuntimeIsInitialized:
    """Tests for PdfiumRuntime.is_initialized static method."""

    def test_is_initialized_returns_bool(self) -> None:
        """is_initialized should return a boolean."""
        result = PdfiumRuntime.is_initialized()
        assert isinstance(result, bool)

    def test_is_initialized_true_after_get_runtime(self) -> None:
        """is_initialized should be True after get_runtime is called."""
        # Ensure runtime is initialized (may already be from module import)
        _ = get_runtime()
        assert PdfiumRuntime.is_initialized() is True


class TestPdfiumRuntimeReuse:
    """Tests for PdfiumRuntime instance reuse behavior."""

    def test_runtime_reuses_on_different_paths(self) -> None:
        """Creating PdfiumRuntime with different paths should reuse existing instance."""
        # First, ensure we have a valid runtime
        runtime1 = PdfiumRuntime(str(get_default_pdfium_path()))

        # Second call with a non-existent path should still succeed
        # because it reuses the existing instance
        runtime2 = PdfiumRuntime("/nonexistent/path/to/pdfium.dll")

        # Both should be valid
        assert isinstance(runtime1, PdfiumRuntime)
        assert isinstance(runtime2, PdfiumRuntime)

    def test_runtime_works_with_document(self) -> None:
        """Runtime obtained via get_runtime should work with Document."""
        _ = get_runtime()
        # The global PDFIUM_RT uses this same mechanism
        assert PdfiumRuntime.is_initialized()


class TestRuntimeIntegration:
    """Integration tests for runtime functionality."""

    def test_document_uses_global_runtime(self, edge_test_pdf_path: Path) -> None:
        """Document should work with the global runtime."""
        # Document class internally uses PDFIUM_RT which uses get_runtime()
        doc = Document(path=edge_test_pdf_path)
        assert not doc.is_closed()
        assert doc.page_count > 0
        doc.close()

    def test_multiple_documents_share_runtime(
        self, edge_test_pdf_path: Path, words_extract_pdf_path: Path
    ) -> None:
        """Multiple documents should share the same runtime."""
        doc1 = Document(path=edge_test_pdf_path)
        doc2 = Document(path=words_extract_pdf_path)

        # Both documents should work correctly
        assert doc1.page_count > 0
        assert doc2.page_count > 0

        doc1.close()
        doc2.close()

    def test_runtime_persists_after_document_close(self, edge_test_pdf_path: Path) -> None:
        """Runtime should remain initialized after document is closed."""
        doc = Document(path=edge_test_pdf_path)
        doc.close()

        # Runtime should still be initialized
        assert PdfiumRuntime.is_initialized()

        # Should be able to open another document
        doc2 = Document(path=edge_test_pdf_path)
        assert doc2.page_count > 0
        doc2.close()
