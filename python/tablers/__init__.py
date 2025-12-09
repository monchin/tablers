import platform
from pathlib import Path
from typing import Final

from .tablers import Document as RsDoc
from .tablers import PdfiumRuntime

SYSTEM: Final = platform.system()

if SYSTEM == "Windows":
    PDFIUM_RT = PdfiumRuntime(str(Path(__file__).parent / "pdfium.dll"))


__doc__ = tablers.__doc__
if hasattr(tablers, "__all__"):
    __all__ = tablers.__all__


class Document:
    def __init__(
        self,
        path: Path | str | None = None,
        bytes: bytes | None = None,
        password: str | None = None,
    ):
        self.doc = RsDoc(
            PDFIUM_RT, path=str(path) if path is not None else None, bytes=bytes, password=password
        )

    @property
    def page_count(self) -> int:
        return self.doc.page_count()
