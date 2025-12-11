from pathlib import Path

class PdfiumRuntime:
    def __init__(self, dll_path: Path | str): ...

class Document:
    def __init__(
        self,
        pdfium_rt: PdfiumRuntime,
        path: Path | str | None = None,
        bytes: bytes | None = None,
        password: str | None = None,
    ): ...
    def page_count(self) -> int: ...
    def get_page(self, page_num: int) -> Page: ...
    def pages(self) -> list[Page]: ...

class Page:
    width: float
    height: float
    page_index: int
    def is_valid(self) -> bool: ...
