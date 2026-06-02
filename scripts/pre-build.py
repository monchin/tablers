"""Pre-build hook: select the correct PDFium binary for the target platform.

Libraries live directly in ``python/tablers/``::

    pdfium.dll               – Windows
    libpdfium.dylib          – macOS
    libpdfium.so.1           – Linux x86_64
    libpdfium-aarch64.so.1   – Linux aarch64

This script removes (moves to a staging area) every library that does **not**
belong to the current build target, leaving only the correct one in place.

For Linux aarch64 builds the file ``libpdfium-aarch64.so.1`` is renamed to
``libpdfium.so.1`` so that the runtime path remains consistent.

The companion ``post-build.py`` script reverses these operations.
"""

import json
import os
import platform
import shutil
import sys
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
# Target detection
# ---------------------------------------------------------------------------
SYSTEM: Final = os.environ.get("BUILD_TARGET", platform.system())
MACHINE: Final = os.environ.get("BUILD_ARCH", platform.machine())

# ---------------------------------------------------------------------------
# Mapping: platform → files that should REMAIN in python/tablers/
# ---------------------------------------------------------------------------
_STAY_MAP: Final[dict[str, list[str]]] = {
    "Windows": ["pdfium.dll"],
    "Darwin": ["libpdfium.dylib"],
    "Linux": [],  # handled separately (depends on arch)
}

# All known library files — loaded from shared config.
_BUILD_CONFIG: Final = json.loads((SCRIPTS_DIR / "build_libs.json").read_text())
_ALL_LIBS: Final = _BUILD_CONFIG["all_libs"]


def _linux_stay_files() -> list[str]:
    """Return the list of files that should stay for a Linux build."""
    if MACHINE == "aarch64":
        return ["libpdfium-aarch64.so.1"]
    # Default to x86_64
    return ["libpdfium.so.1"]


if __name__ == "__main__":
    print(f"[pre-build] target: os={SYSTEM}, arch={MACHINE}")

    # Determine which files stay
    if SYSTEM == "Linux":
        stay = _linux_stay_files()
    else:
        stay = _STAY_MAP[SYSTEM]

    # Files to move out
    to_move = [f for f in _ALL_LIBS if f not in stay]

    # Prepare staging
    STAGING_DIR.mkdir(parents=True, exist_ok=True)

    moved: list[str] = []
    for fname in to_move:
        src = SRC_ROOT / fname
        if src.exists():
            shutil.move(str(src), str(STAGING_DIR / fname))
            moved.append(fname)

    print(f"[pre-build] moved to staging: {moved}")

    # For Linux aarch64: rename the aarch64 lib to the canonical name
    if SYSTEM == "Linux" and MACHINE == "aarch64":
        aarch64_src = SRC_ROOT / "libpdfium-aarch64.so.1"
        canonical = SRC_ROOT / "libpdfium.so.1"
        if aarch64_src.exists():
            shutil.move(str(aarch64_src), str(canonical))
            print("[pre-build] renamed libpdfium-aarch64.so.1 -> libpdfium.so.1")

    # Verify the expected library exists after all moves/renames
    if SYSTEM == "Linux":
        expected_path = SRC_ROOT / "libpdfium.so.1"
    else:
        expected_name = _STAY_MAP[SYSTEM][0]
        expected_path = SRC_ROOT / expected_name

    if not expected_path.exists():
        print(
            f"[pre-build] ERROR: expected library {expected_path} not found!",
            file=sys.stderr,
        )
        sys.exit(1)

    print("[pre-build] done")
