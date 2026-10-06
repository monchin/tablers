"""Pre-build hook: select the correct PDFium binary for the target platform.

Libraries live directly in ``python/tablers/``::

    pdfium.dll               – Windows x64
    libpdfium.dylib          – macOS ARM64 (Apple Silicon)
    libpdfium-x86_64.dylib   – macOS x86_64 (Intel)
    libpdfium.so.1           – Linux x86_64
    libpdfium-aarch64.so.1   – Linux aarch64

This script removes (moves to a staging area) every library that does **not**
belong to the current build target, leaving only the correct one in place.

Each platform has one *canonical* runtime name (``pdfium.dll``,
``libpdfium.dylib``, ``libpdfium.so.1``). If the build target uses a
non-default architecture for its OS (Linux aarch64, macOS x86_64), the
matching arch-suffixed library is renamed to the canonical name so that the
runtime path remains consistent.

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
# Platform mapping
# ---------------------------------------------------------------------------
# Canonical runtime library name for each supported OS (the name pdfium-render
# loads at runtime).
_CANONICAL_LIB: Final[dict[str, str]] = {
    "Windows": "pdfium.dll",
    "Darwin": "libpdfium.dylib",
    "Linux": "libpdfium.so.1",
}

# Arch-suffixed library that must be renamed to the canonical name when
# building for a non-default architecture of an OS.
_ARCH_LIB: Final[dict[tuple[str, str], str]] = {
    ("Linux", "aarch64"): "libpdfium-aarch64.so.1",
    ("Darwin", "x86_64"): "libpdfium-x86_64.dylib",
}

# All known library files — loaded from shared config.
_BUILD_CONFIG: Final = json.loads((SCRIPTS_DIR / "build_libs.json").read_text())
_ALL_LIBS: Final = _BUILD_CONFIG["all_libs"]


if __name__ == "__main__":
    print(f"[pre-build] target: os={SYSTEM}, arch={MACHINE}")

    if SYSTEM not in _CANONICAL_LIB:
        print(f"[pre-build] ERROR: unsupported system: {SYSTEM}", file=sys.stderr)
        sys.exit(1)

    canonical = _CANONICAL_LIB[SYSTEM]
    arch_lib = _ARCH_LIB.get((SYSTEM, MACHINE))

    # Files that should stay: the arch-specific one when building for a
    # non-default arch, otherwise the canonical one.
    stay = [arch_lib] if arch_lib else [canonical]

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

    # For non-default arch builds: rename the arch-suffixed lib to the
    # canonical runtime name.
    if arch_lib:
        arch_src = SRC_ROOT / arch_lib
        canonical_path = SRC_ROOT / canonical
        if arch_src.exists():
            shutil.move(str(arch_src), str(canonical_path))
            print(f"[pre-build] renamed {arch_lib} -> {canonical}")

    # Verify the expected library exists after all moves/renames
    expected_path = SRC_ROOT / canonical
    if not expected_path.exists():
        print(
            f"[pre-build] ERROR: expected library {expected_path} not found!",
            file=sys.stderr,
        )
        sys.exit(1)

    print("[pre-build] done")
