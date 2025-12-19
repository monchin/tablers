import sys
from pathlib import Path
from typing import Literal, TypeAlias, TypedDict

if sys.version_info < (3, 11):
    from typing_extensions import Unpack
else:
    from typing import Unpack

Point: TypeAlias = tuple[float, float]
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
    def extract_objects(self) -> None: ...
    def clear(self): ...
    @property
    def objects(self) -> Objects | None: ...

class Objects:
    rects: list[Rect]
    lines: list[Line]

class Rect:
    bbox: BBox
    fill_color: tuple[int, int, int, int]
    stroke_color: tuple[int, int, int, int]
    stroke_width: float

class Line:
    line_type: Literal["straight", "curve"]
    points: list[Point]
    color: tuple[int, int, int, int]
    width: float

class Edge:
    orietation: Literal["h", "v"]
    x1: float
    y1: float
    x2: float
    y2: float
    width: float
    color: tuple[int, int, int, int]

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
    min_words_vertical: int
    min_words_horizontal: int
    intersection_x_tolerance: float
    intersection_y_tolerance: float

def find_tables(
    page: Page,
    extract_text: bool,
    bottom_origin: bool = False,
    **kwargs: Unpack[TfSettingItems],
) -> tuple[list[BBox], list[Table]]: ...
