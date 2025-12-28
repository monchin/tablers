from __future__ import annotations

import sys
from collections.abc import Iterator
from pathlib import Path
from typing import Literal, TypeAlias, TypedDict

if sys.version_info < (3, 11):
    from typing_extensions import Unpack
else:
    from typing import Unpack

Point: TypeAlias = tuple[float, float]
BBox: TypeAlias = tuple[float, float, float, float]
Color: TypeAlias = tuple[int, int, int, int]  # RGBA, each 0~255

__version__: str

class PdfiumRuntime:
    def __init__(self, dll_path: Path | str): ...

class PageIterator(Iterator[Page]):
    """Iterator over PDF pages (memory efficient for large PDFs)"""
    def __iter__(self) -> PageIterator: ...
    def __next__(self) -> Page: ...

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
    def pages(self) -> PageIterator: ...
    def close(self) -> None: ...
    def is_closed(self) -> bool: ...
    def __iter__(self) -> PageIterator: ...

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
    chars: list[Char]

class Rect:
    bbox: BBox
    fill_color: Color
    stroke_color: Color
    stroke_width: float

class Line:
    line_type: Literal["straight", "curve"]
    points: list[Point]
    color: Color
    width: float

class Char:
    unicode_char: str | None
    bbox: BBox
    rotation_degrees: float
    upright: bool

class Edge:
    orietation: Literal["h", "v"]
    x1: float
    y1: float
    x2: float
    y2: float
    width: float
    color: Color

class TableCell:
    bbox: BBox
    text: str

class Table:
    bbox: BBox
    cells: list[TableCell]

class WordsExtractSettingsItems(TypedDict, total=False):
    x_tolerance: float
    y_tolerance: float
    keep_blank_chars: bool
    use_text_flow: bool
    text_read_in_clockwise: bool
    split_at_punctuation: Literal["all"] | str | None
    expand_ligatures: bool

class WordsExtractSettings:
    """Settings for text/word extraction."""

    x_tolerance: float
    y_tolerance: float
    keep_blank_chars: bool
    use_text_flow: bool
    text_read_in_clockwise: bool
    split_at_punctuation: str | None
    expand_ligatures: bool

    def __init__(self, **kwargs: Unpack[WordsExtractSettingsItems]) -> None: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...

class TfSettingItems(TypedDict, total=False):
    vertical_strategy: Literal["lines", "lines_strict", "text"]
    horizontal_strategy: Literal["lines", "lines_strict", "text"]
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
    text_x_tolerance: float
    text_y_tolerance: float
    text_keep_blank_chars: bool
    text_use_text_flow: bool
    text_read_in_clockwise: bool
    text_split_at_punctuation: Literal["all"] | str | None
    text_expand_ligatures: bool

class TfSettings:
    """Settings for table finding."""

    vertical_strategy: Literal["lines", "lines_strict", "text"]
    horizontal_strategy: Literal["lines", "lines_strict", "text"]
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
    text_settings: WordsExtractSettings
    text_x_tolerance: float
    text_y_tolerance: float
    text_keep_blank_chars: bool
    text_use_text_flow: bool
    text_read_in_clockwise: bool
    text_split_at_punctuation: str | None
    text_expand_ligatures: bool

    def __init__(self, **kwargs: Unpack[TfSettingItems]) -> None: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...

def find_all_cells_bboxes(
    page: Page, tf_settings: TfSettings | None = None, **kwargs
) -> list[BBox]: ...
def find_tables_from_cells(
    cells: list[BBox],
    extract_text: bool,
    page: Page | None = None,
    we_settings: WordsExtractSettings | None = None,
    **kwargs: Unpack[TfSettingItems],
) -> list[Table]: ...
def find_tables(
    page: Page,
    extract_text: bool,
    tf_settings: TfSettings | None = None,
    **kwargs: Unpack[TfSettingItems],
) -> list[Table]: ...
def get_edges(page: Page, settings: TfSettingItems | None = None) -> dict[str, list[Edge]]: ...
