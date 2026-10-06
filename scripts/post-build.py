"""Post-build hook: restore PDFium libraries after a wheel build.

Reverses the operations performed by ``pre-build.py``:
1. For non-default arch builds (Linux aarch64, macOS x86_64), renames the
   canonical library back to its arch-suffixed name.
2. Moves staged libraries back into ``python/tablers/``.
3. Removes the staging directory.
"""

import json
import os
import platform
import shutil
from pathlib import Path
from typing import Final

# ---------------------------------------------------------------------------
# Paths
# ---------------------------------------------------------------------------
SCRIPTS_DIR: Final = Path(__file__).parent.absolute()
PRJ_ROOT: Final = SCRIPTS_DIR.parent
SRC_ROOT: Final = PRJ_ROOT / "python" / "tablers"
STAGING_DIR: Final = SCRIPTS_DIR / "_staging"

# ---------------------------------------------------------------------------
# Target detection (must match pre-build)
# ---------------------------------------------------------------------------
SYSTEM: Final = os.environ.get("BUILD_TARGET", platform.system())
MACHINE: Final = os.environ.get("BUILD_ARCH", platform.machine())

# ---------------------------------------------------------------------------
# Platform mapping (must match pre-build)
# ---------------------------------------------------------------------------
_CANONICAL_LIB: Final[dict[str, str]] = {
    "Windows": "pdfium.dll",
    "Darwin": "libpdfium.dylib",
    "Linux": "libpdfium.so.1",
}

_ARCH_LIB: Final[dict[tuple[str, str], str]] = {
    ("Linux", "aarch64"): "libpdfium-aarch64.so.1",
    ("Darwin", "x86_64"): "libpdfium-x86_64.dylib",
}

# All known library files — loaded from shared config.
_BUILD_CONFIG: Final = json.loads((SCRIPTS_DIR / "build_libs.json").read_text())
_ALL_LIBS: Final = _BUILD_CONFIG["all_libs"]


if __name__ == "__main__":
    arch_lib = _ARCH_LIB.get((SYSTEM, MACHINE))

    # For non-default arch builds: rename back from canonical name
    if arch_lib:
        canonical = SRC_ROOT / _CANONICAL_LIB[SYSTEM]
        arch_dst = SRC_ROOT / arch_lib
        if canonical.exists() and not arch_dst.exists():
            shutil.move(str(canonical), str(arch_dst))
            print(f"[post-build] renamed {_CANONICAL_LIB[SYSTEM]} -> {arch_lib}")
        else:
            print(
                f"[post-build] arch rename skipped: "
                f"canonical={canonical.exists()}, dst={arch_dst.exists()}"
            )

    # Move staged files back
    restored: list[str] = []
    for fname in _ALL_LIBS:
        staged = STAGING_DIR / fname
        if staged.exists():
            shutil.move(str(staged), str(SRC_ROOT / fname))
            restored.append(fname)

    print(f"[post-build] restored: {restored}")

    # Clean up staging
    if STAGING_DIR.exists():
        shutil.rmtree(STAGING_DIR)

    print("[post-build] done")
