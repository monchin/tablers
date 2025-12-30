import platform
import shutil
from pathlib import Path
from typing import Final

PLATFORM: Final = platform.system()
PRJ_ROOT: Final = Path(__file__).parent.parent.absolute()
SRC_ROOT: Final = PRJ_ROOT / "python" / "tablers"
CUR_DIR: Final = Path(__file__).parent.absolute()
DLL_NO_NEED_MAP: Final[dict[str, list[str]]] = {
    "Windows": ["libpdfium.so.1", "libpdfium.dylib"],
    "Linux": ["pdfium.dll", "libpdfium.dylib"],
    "Darwin": ["pdfium.dll", "libpdfium.so.1"],
}
if __name__ == "__main__":
    dll_no_need = DLL_NO_NEED_MAP[PLATFORM]
    for dll in dll_no_need:
        dll_path = CUR_DIR / dll
        if dll_path.exists():
            shutil.move(dll_path, SRC_ROOT)
