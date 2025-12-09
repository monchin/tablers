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
