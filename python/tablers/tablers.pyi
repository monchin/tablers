from pathlib import Path
from typing import TypeAlias, TypedDict

BBox: TypeAlias = tuple[float, float, float, float]

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
    def close(self) -> None: ...
    def is_closed(self) -> bool: ...

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

class TableCell:
    bbox: BBox
    text: str

class Table:
    bbox: BBox
    cells: list[TableCell]

class TfSettingItems(TypedDict, total=False):
    vertical_strategy: str
    horizontal_strategy: str
    snap_x_tolerance: float
    snap_y_tolerance: float
    join_x_tolerance: float
    join_y_tolerance: float
    edge_min_length: float
    edge_min_length_prefilter: float
    intersection_x_tolerance: float
    intersection_y_tolerance: float

def find_tables(
    page: Page,
    extract_text: bool,
    bottom_origin: bool = False,
    **kwargs: TfSettingItems,
) -> tuple[list[BBox], list[Table]]: ...
