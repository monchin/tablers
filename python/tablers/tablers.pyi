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
        bottom_origin: bool = False,
    ): ...
    def page_count(self) -> int: ...
    def get_page(self, page_num: int) -> Page: ...
    def pages(self) -> list[Page]: ...

class Page:
    width: float
    height: float
    def is_valid(self) -> bool: ...
    def extract_edges(self) -> None: ...
    @property
    def edges(self) -> dict[str, list[Edge]]: ...

class Edge:
    x1: float
    y1: float
    x2: float
    y2: float
    width: float
    color: tuple[int, int, int, int]
    edge_type: str
