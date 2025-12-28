"""
Shared pytest fixtures for tablers tests.
"""

from collections.abc import Generator
from pathlib import Path

import pytest
from tablers import Document

# Path to test data directory
TEST_DATA_DIR = Path(__file__).parent.parent / "data"


@pytest.fixture
def edge_test_pdf_path() -> Path:
    """Return path to the edge-test.pdf file."""
    return TEST_DATA_DIR / "edge-test.pdf"


@pytest.fixture
def words_extract_pdf_path() -> Path:
    """Return path to the words-extract.pdf file."""
    return TEST_DATA_DIR / "words-extract.pdf"


@pytest.fixture
def edge_test_doc(edge_test_pdf_path: Path) -> Generator[Document, None, None]:
    """Open and return a Document for edge-test.pdf, closing it after the test."""
    doc = Document(path=edge_test_pdf_path)
    yield doc
    if not doc.is_closed():
        doc.close()


@pytest.fixture
def words_extract_doc(words_extract_pdf_path: Path) -> Generator[Document, None, None]:
    """Open and return a Document for words-extract.pdf, closing it after the test."""
    doc = Document(path=words_extract_pdf_path)
    yield doc
    if not doc.is_closed():
        doc.close()


@pytest.fixture
def edge_test_pdf_bytes(edge_test_pdf_path: Path) -> bytes:
    """Return the content of edge-test.pdf as bytes."""
    return edge_test_pdf_path.read_bytes()


@pytest.fixture
def words_extract_pdf_bytes(words_extract_pdf_path: Path) -> bytes:
    """Return the content of words-extract.pdf as bytes."""
    return words_extract_pdf_path.read_bytes()
