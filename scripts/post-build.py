"""Post-build hook: restore PDFium libraries after a wheel build.

Reverses the operations performed by ``pre-build.py``:
1. Moves staged libraries back into ``python/tablers/``.
2. For Linux aarch64 builds, renames ``libpdfium.so.1`` back to
   ``libpdfium-aarch64.so.1``.
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

# All known library files — loaded from shared config.
_BUILD_CONFIG: Final = json.loads((SCRIPTS_DIR / "build_libs.json").read_text())
_ALL_LIBS: Final = _BUILD_CONFIG["all_libs"]


if __name__ == "__main__":
    # For Linux aarch64: rename back from canonical name
    if SYSTEM == "Linux" and MACHINE == "aarch64":
        canonical = SRC_ROOT / "libpdfium.so.1"
        aarch64_dst = SRC_ROOT / "libpdfium-aarch64.so.1"
        if canonical.exists() and not aarch64_dst.exists():
            shutil.move(str(canonical), str(aarch64_dst))
            print("[post-build] renamed libpdfium.so.1 -> libpdfium-aarch64.so.1")
        else:
            print(
                f"[post-build] aarch64 rename skipped: "
                f"canonical={canonical.exists()}, dst={aarch64_dst.exists()}"
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
