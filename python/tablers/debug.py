"""
This module is copied from `pdfplumber/display.py` and modified to work with `tablers`. It provides
utilities for visualizing PDF pages and debugging table detection by drawing detected tables, edges,
and intersections on the page image.

To use this module, you need `pillow` and `pypdfium2` installed. You can install them via pip:
```bash
pip install tablers[debug]
```
Or if you are developing `tablers` locally, you can install the debug dependencies with:
```bash
pdm sync -G debug
```

Example usage:
```python
from tablers import Document
from tablers.debug import PageImage

with Document("path/to/your/document.pdf") as doc:
    page = doc.get_page(0)
    im = PageImage(page)
    im.debug_tablefinder()
    im.show() # or `im.save("output.png")` to save the image to a file
```

**Password-protected PDFs:** PageImage rendering supports only documents without a password.
For password-protected PDFs, use ``Document.save_to_bytes()`` to obtain a decrypted copy,
then open it with ``Document(bytes=...)`` and pass the resulting page to PageImage.

Please note that the `debug_tablefinder` method can be customized with various settings for table
detection, which can be passed as keyword arguments. See more details in the documents.
"""

from __future__ import annotations

from io import BytesIO
from pathlib import Path
from typing import TYPE_CHECKING, TypeAlias

import PIL.Image
import PIL.ImageDraw
import pypdfium2

from .page import Page
from .tablers import Table, TfSettings
from .typing import BBox, Point, TfSettingItems

if TYPE_CHECKING:
    import sys

    if sys.version_info >= (3, 11):
        from types import Unpack
    else:
        from typing_extensions import Unpack

T_color: TypeAlias = tuple[int, ...]
T_line: TypeAlias = tuple[Point, Point]


class Colors:
    """Named RGBA color constants for drawing."""

    RED = (255, 0, 0)
    GREEN = (0, 255, 0)
    BLUE = (0, 0, 255)
    TRANSPARENT = (0, 0, 0, 0)


DEFAULT_FILL = Colors.BLUE + (50,)
DEFAULT_STROKE = Colors.RED + (200,)
DEFAULT_STROKE_WIDTH = 1
DEFAULT_RESOLUTION = 72


def get_page_image(
    stream: BytesIO,
    page_ix: int,
    resolution: int | float,
    password: str | None,
    antialias: bool = False,
) -> PIL.Image.Image:
    stream.seek(0)
    pdfium_doc = pypdfium2.PdfDocument(stream, password=password)
    try:
        pdfium_page = pdfium_doc.get_page(page_ix)
        img: PIL.Image.Image = pdfium_page.render(
            scale=resolution / 72,
            no_smoothtext=not antialias,
            no_smoothpath=not antialias,
            no_smoothimage=not antialias,
            prefer_bgrx=True,
        ).to_pil()
        return img.convert("RGB")
    finally:
        pdfium_doc.close()


class PageImage:
    def __init__(
        self,
        page: Page,
        original: PIL.Image.Image | None = None,
        resolution: int | float = DEFAULT_RESOLUTION,
        antialias: bool = False,
    ):
        self.page = page
        self.resolution = resolution

        if original is None:
            if page.doc.is_closed():
                raise RuntimeError("Cannot convert closed PDF document to image")
            doc_stream = page.doc.save_to_bytes()
            self.original = get_page_image(
                stream=BytesIO(doc_stream),
                page_ix=page.page_idx,
                resolution=resolution,
                antialias=antialias,
                password=None,
            )
        else:
            self.original = original

        self.scale = self.original.size[0] / page.width
        self.bbox: BBox = (0, 0, page.width, page.height)
        self.reset()

    def _reproject_bbox(self, bbox: BBox) -> tuple[int, int, int, int]:
        x1, y1, x2, y2 = bbox
        _x1, _y1 = self._reproject((x1, y1))
        _x2, _y2 = self._reproject((x2, y2))
        return (_x1, _y1, _x2, _y2)

    def _reproject(self, coord: Point) -> tuple[int, int]:
        """
        Given an (x, y) coordinate in page units, return the corresponding
        (x, y) coordinate in image pixels.
        """
        x1, y1 = coord
        _x1 = (x1 - self.bbox[0]) * self.scale
        _y1 = (y1 - self.bbox[1]) * self.scale
        return (int(_x1), int(_y1))

    def reset(self) -> PageImage:
        self.annotated = PIL.Image.new("RGB", self.original.size)
        self.annotated.paste(self.original)
        self.draw = PIL.ImageDraw.Draw(self.annotated, "RGBA")
        return self

    def save(
        self,
        dest: str | Path | BytesIO,
        format: str = "PNG",
        quantize: bool = True,
        colors: int = 256,
        bits: int = 8,
        **kwargs,
    ) -> None:
        if quantize:
            out = self.annotated.quantize(colors, method=PIL.Image.Quantize.FASTOCTREE).convert("P")
        else:
            out = self.annotated
        out.save(
            dest,
            format=format,
            bits=bits,
            dpi=(self.resolution, self.resolution),
            **kwargs,
        )

    def copy(self) -> PageImage:
        return self.__class__(self.page, self.original)

    def draw_line(
        self,
        points: T_line,
        stroke: T_color = DEFAULT_STROKE,
        stroke_width: int = DEFAULT_STROKE_WIDTH,
    ) -> PageImage:
        self.draw.line(list(map(self._reproject, points)), fill=stroke, width=stroke_width)
        return self

    def draw_lines(
        self,
        list_of_lines: list[T_line],
        stroke: T_color = DEFAULT_STROKE,
        stroke_width: int = DEFAULT_STROKE_WIDTH,
    ) -> PageImage:
        for line in list_of_lines:
            self.draw_line(line, stroke=stroke, stroke_width=stroke_width)
        return self

    def draw_vline(
        self,
        location: float,
        stroke: T_color = DEFAULT_STROKE,
        stroke_width: int = DEFAULT_STROKE_WIDTH,
    ) -> PageImage:
        points = (location, self.bbox[1], location, self.bbox[3])
        self.draw.line(self._reproject_bbox(points), fill=stroke, width=stroke_width)
        return self

    def draw_vlines(
        self,
        locations: list[float],
        stroke: T_color = DEFAULT_STROKE,
        stroke_width: int = DEFAULT_STROKE_WIDTH,
    ) -> PageImage:
        for location in locations:
            self.draw_vline(location, stroke=stroke, stroke_width=stroke_width)
        return self

    def draw_hline(
        self,
        location: float,
        stroke: T_color = DEFAULT_STROKE,
        stroke_width: int = DEFAULT_STROKE_WIDTH,
    ) -> PageImage:
        points = (self.bbox[0], location, self.bbox[2], location)
        self.draw.line(self._reproject_bbox(points), fill=stroke, width=stroke_width)
        return self

    def draw_hlines(
        self,
        locations: list[float],
        stroke: T_color = DEFAULT_STROKE,
        stroke_width: int = DEFAULT_STROKE_WIDTH,
    ) -> PageImage:
        for location in locations:
            self.draw_hline(location, stroke=stroke, stroke_width=stroke_width)
        return self

    def draw_rect(
        self,
        bbox: BBox,
        fill: T_color = DEFAULT_FILL,
        stroke: T_color = DEFAULT_STROKE,
        stroke_width: int = DEFAULT_STROKE_WIDTH,
    ) -> PageImage:

        x1, y1, x2, y2 = bbox
        half = stroke_width / 2
        mid_x = (x1 + x2) / 2
        mid_y = (y1 + y2) / 2
        x1_adj = min(x1 + half, mid_x)
        y1_adj = min(y1 + half, mid_y)
        x2_adj = max(x2 - half, mid_x)
        y2_adj = max(y2 - half, mid_y)

        fill_bbox = self._reproject_bbox((x1_adj, y1_adj, x2_adj, y2_adj))
        self.draw.rectangle(fill_bbox, fill, Colors.TRANSPARENT)

        if stroke_width > 0:
            segments = [
                ((x1_adj, y1_adj), (x2_adj, y1_adj)),
                ((x1_adj, y2_adj), (x2_adj, y2_adj)),
                ((x1_adj, y1_adj), (x1_adj, y2_adj)),
                ((x2_adj, y1_adj), (x2_adj, y2_adj)),
            ]
            self.draw_lines(segments, stroke=stroke, stroke_width=stroke_width)
        return self

    def draw_rects(
        self,
        rects: list[BBox],
        fill: T_color = DEFAULT_FILL,
        stroke: T_color = DEFAULT_STROKE,
        stroke_width: int = DEFAULT_STROKE_WIDTH,
    ) -> PageImage:
        for rect in rects:
            self.draw_rect(rect, fill=fill, stroke=stroke, stroke_width=stroke_width)
        return self

    def draw_circle(
        self,
        center: Point,
        radius: int = 5,
        fill: T_color = DEFAULT_FILL,
        stroke: T_color = DEFAULT_STROKE,
    ) -> PageImage:
        cx, cy = center
        bbox = (cx - radius, cy - radius, cx + radius, cy + radius)
        self.draw.ellipse(self._reproject_bbox(bbox), fill, stroke)
        return self

    def draw_circles(
        self,
        circles: list[Point],
        radius: int = 5,
        fill: T_color = DEFAULT_FILL,
        stroke: T_color = DEFAULT_STROKE,
    ) -> PageImage:
        for circle in circles:
            self.draw_circle(circle, radius=radius, fill=fill, stroke=stroke)
        return self

    def debug_table(
        self,
        table: Table,
        fill: T_color = DEFAULT_FILL,
        stroke: T_color = DEFAULT_STROKE,
        stroke_width: int = 1,
    ) -> PageImage:
        """Outline all cells of a table."""
        cell_bboxes = [cell.bbox for cell in table.cells]
        self.draw_rects(cell_bboxes, fill=fill, stroke=stroke, stroke_width=stroke_width)
        return self

    def debug_tablefinder(
        self,
        tf_settings: TfSettings | None = None,
        **kwargs: Unpack[TfSettingItems],
    ) -> PageImage:
        """Draw detected tables and edges on the page image."""
        from . import find_tables, get_edges, get_intersections_from_edges

        tables = find_tables(self.page, extract_text=False, tf_settings=tf_settings, **kwargs)
        for table in tables:
            self.debug_table(table)

        edges_dict = get_edges(self.page, tf_settings=tf_settings, **kwargs)
        edge_lines = [((e.x1, e.y1), (e.x2, e.y2)) for edges in edges_dict.values() for e in edges]
        intersections = get_intersections_from_edges(
            edges_dict.get("h", []), edges_dict.get("v", [])
        )
        self.draw_lines(edge_lines, stroke_width=1)
        self.draw_circles(
            list(intersections.keys()),
            radius=3,
            fill=Colors.TRANSPARENT,
            stroke=Colors.BLUE + (200,),
        )
        return self

    def _repr_png_(self) -> bytes:
        b = BytesIO()
        self.save(b, "PNG")
        return b.getvalue()

    def show(self) -> None:  # pragma: no cover
        self.annotated.show()
