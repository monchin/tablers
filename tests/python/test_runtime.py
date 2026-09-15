"""
Tests for PdfiumRuntime and related functionality.
"""

import os
import platform
import subprocess
import sys
import textwrap
import threading
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

from tablers import (
    Document,
    PdfiumRuntime,
    find_tables,
    get_default_pdfium_path,
    get_runtime,
)


def _minimal_pdf_bytes() -> bytes:
    """Build a synthetic one-page PDF without relying on a PDF test dependency."""
    objects = (
        b"<< /Type /Catalog /Pages 2 0 R >>",
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 300] /Contents 4 0 R >>",
        b"<< /Length 0 >>\nstream\n\nendstream",
    )
    content = bytearray(b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n")
    offsets = [0]
    for number, body in enumerate(objects, start=1):
        offsets.append(len(content))
        content.extend(f"{number} 0 obj\n".encode())
        content.extend(body)
        content.extend(b"\nendobj\n")
    xref_offset = len(content)
    content.extend(f"xref\n0 {len(objects) + 1}\n".encode())
    content.extend(b"0000000000 65535 f \n")
    for offset in offsets[1:]:
        content.extend(f"{offset:010d} 00000 n \n".encode())
    content.extend(
        f"trailer\n<< /Size {len(objects) + 1} /Root 1 0 R >>\n"
        f"startxref\n{xref_offset}\n%%EOF\n".encode()
    )
    return bytes(content)


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
        """Multiple calls should create valid handles on the current thread."""
        runtime1 = get_runtime()
        runtime2 = get_runtime()
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

    def test_concurrent_threads_repeatedly_open_synthetic_documents(self, tmp_path: Path) -> None:
        """Independent thread runtimes should survive sustained Pdfium contention."""
        pdf_bytes = _minimal_pdf_bytes()
        pdf_path = tmp_path / "thread-contention.pdf"
        pdf_path.write_bytes(pdf_bytes)
        worker_count = 8
        iterations = 20
        ready = threading.Barrier(worker_count)

        def hammer_runtime(worker_index: int) -> tuple[int, int]:
            """Repeatedly open path and byte documents from one synchronized thread."""
            runtime = get_runtime()
            assert isinstance(runtime, PdfiumRuntime)
            ready.wait(timeout=10)
            completed = 0
            for iteration in range(iterations):
                if (worker_index + iteration) % 2 == 0:
                    document = Document(path=pdf_path)
                else:
                    document = Document(bytes=pdf_bytes)
                with document:
                    page = document.get_page(0)
                    assert document.page_count == 1
                    assert page.width == 200
                    assert page.height == 300
                    page.extract_objects()
                    assert find_tables(page) == []
                    completed += 1
            return threading.get_ident(), completed

        with ThreadPoolExecutor(max_workers=worker_count) as executor:
            results = list(executor.map(hammer_runtime, range(worker_count)))

        assert len({thread_id for thread_id, _ in results}) == worker_count
        assert all(completed == iterations for _, completed in results)

    def test_replacement_worker_threads_get_fresh_runtime_handles(self, tmp_path: Path) -> None:
        """Short-lived replacement workers should each create usable documents."""
        pdf_path = tmp_path / "replacement-workers.pdf"
        pdf_path.write_bytes(_minimal_pdf_bytes())
        results: list[tuple[int, int]] = []

        def inspect_document() -> None:
            """Read the synthetic document entirely inside one short-lived thread."""
            with Document(path=pdf_path) as document:
                results.append((threading.get_ident(), document.page_count))

        for _ in range(20):
            worker = threading.Thread(target=inspect_document)
            worker.start()
            worker.join(timeout=10)
            assert not worker.is_alive()

        assert len(results) == 20
        assert all(page_count == 1 for _, page_count in results)

    def test_first_import_on_worker_thread_shuts_down_cleanly(self, tmp_path: Path) -> None:
        """A fresh process should not retain an import-worker runtime until shutdown."""
        pdf_path = tmp_path / "worker-first-import.pdf"
        pdf_path.write_bytes(_minimal_pdf_bytes())
        package_root = Path(sys.modules[Document.__module__].__file__).resolve().parent.parent
        environment = os.environ.copy()
        environment["PYTHONPATH"] = os.pathsep.join(
            filter(None, (str(package_root), environment.get("PYTHONPATH", "")))
        )
        script = textwrap.dedent(
            """
            import sys
            import threading

            results = []

            def inspect_document():
                from tablers import Document

                with Document(path=sys.argv[1]) as document:
                    results.append(document.page_count)

            worker = threading.Thread(target=inspect_document)
            worker.start()
            worker.join(timeout=10)
            if worker.is_alive() or results != [1]:
                raise RuntimeError(f"worker result: {results!r}")
            """
        )

        completed = subprocess.run(
            [sys.executable, "-c", script, str(pdf_path)],
            capture_output=True,
            check=False,
            env=environment,
            text=True,
            timeout=20,
        )

        assert completed.returncode == 0, completed.stderr
        assert "unsendable" not in completed.stderr
