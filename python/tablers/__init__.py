from __future__ import annotations

import platform
from pathlib import Path
from typing import Final

from .tablers import Document as RsDoc
from .tablers import Page, PageIterator, PdfiumRuntime, __version__, find_tables

SYSTEM: Final = platform.system()

if SYSTEM == "Windows":
    PDFIUM_RT = PdfiumRuntime(str(Path(__file__).parent / "pdfium.dll"))
elif SYSTEM == "Linux":
    PDFIUM_RT = PdfiumRuntime(str(Path(__file__).parent / "libpdfium.so"))
elif SYSTEM == "Darwin":
    PDFIUM_RT = PdfiumRuntime(str(Path(__file__).parent / "libpdfium.dylib"))
else:
    raise RuntimeError(f"Unsupported system: {SYSTEM}")


__all__ = ["Document", "Page", "find_tables", "__version__"]


class Document:
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

    @property
    def page_count(self) -> int:
        return self.doc.page_count()

    def get_page(self, page_num: int) -> Page:
        return self.doc.get_page(page_num)

    def pages(self) -> PageIterator:
        return self.doc.pages()

    def close(self) -> None:
        self.doc.close()

    def is_closed(self) -> bool:
        return self.doc.is_closed()
