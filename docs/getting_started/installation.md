# Installation

## Requirements

Before installing Tablers, ensure your system meets the following requirements:

- **Python**: >= 3.10
- **Operating System**:
    - Windows (x64)
    - Linux (x64 / ARM64), glibc >= 2.28 (manylinux_2_28)
    - macOS (ARM64 / Apple Silicon)

## Install with pip

The recommended way to install Tablers is via pip:

```bash
pip install tablers
```

## Optional Dependencies

### Debug / Visualization

The `tablers.debug` module provides tools for visualizing detected tables, edges, and intersection points on a rendered page image. It requires two additional packages:

```bash
pip install tablers[debug]
```

This installs `pillow` and `pypdfium2` alongside Tablers. If these packages are not present, importing `tablers.debug` will raise an `ImportError`.

## Building from Source

If you need to build Tablers from source, follow these steps:

### Prerequisites

```bash
# Install Rust toolchain
# Visit https://rustup.rs/ for installation instructions

# Install uv (recommended)
# See https://docs.astral.sh/uv/getting-started/installation/

# Install build tools
uv tool install maturin
uv tool install pdm
```

### Build Steps

```bash
# Clone the repository
git clone https://github.com/monchin/tablers.git
cd tablers

# Install dependencies
pdm sync

# Build the Rust extension
maturin develop --uv

# Run tests to verify installation
pdm test
```

## Development Notes

### Generating Type Stub Files

During development, if you need to generate or update `.pyi` type stub files for better IDE support and type checking, you can run:

```bash
pdm stub
```

This command requires that you have already installed `uv` and `pdm` as part of the build prerequisites. The generated stub files will provide comprehensive type hints for the Tablers API.

**Important Notes:**

- `.pyi` files are added to `.gitignore` and should **not** be manually edited or committed
- Type stub files are automatically generated during CI and release workflows
- Any manual changes to `.pyi` files will be overwritten when `pdm stub` is run
- **Before committing code**, run `pdm stub` to ensure the generated `.pyi` files are up-to-date for pre-commit hooks to pass successfully

## Verify Installation

After installation, you can verify it was successful:

```python
import tablers
print(tablers.__version__)
```

Or run a simple test:

```python
from tablers import Document, find_tables

# Check if the module loaded correctly
print("tablers installed successfully!")
```

## Troubleshooting

### glibc Version Issues on Linux

If you encounter glibc version errors on Linux, ensure your system glibc version >= 2.28 (manylinux_2_28). You can check with:

```bash
ldd --version
```

### Architecture Issues on macOS

Tablers currently only supports Apple Silicon (ARM64) architecture on macOS. If you're using an Intel Mac, consider building from source and download pdfium binaries from [this project](https://github.com/bblanchon/pdfium-binaries) and replace the pdfium binaries in the `python/tablers` directory.

### Developing on Linux ARM64

Tablers ships pre-built wheels for both x86_64 and ARM64 (aarch64) Linux, so installing via pip works on both architectures. However, **local development on Linux ARM64 requires an extra step**.

The PDFium library for ARM64 is stored as `libpdfium-aarch64.so.1` in the source tree. At runtime the library is expected at `libpdfium.so.1`. Before building or testing locally on an ARM64 machine, run the pre-build hook to rename it:

```bash
# Run the pre-build hook (auto-detects ARM64 and renames the library)
pdm run python scripts/pre-build.py

# Build and test
maturin develop --uv
pdm test

# Restore the original file names when done
pdm run python scripts/post-build.py
```

The pre-build hook auto-detects the platform via `platform.system()` and `platform.machine()`. If the expected library file is missing, the hook will exit with an error. For cross-compilation (building for ARM64 on an x86_64 host), set the target explicitly:

```bash
export BUILD_TARGET=Linux
export BUILD_ARCH=aarch64
pdm run python scripts/pre-build.py
```

Alternatively, you can manually rename the file:

```bash
cd python/tablers
mv libpdfium-aarch64.so.1 libpdfium.so.1
```

### Adding a New Platform

To add support for a new platform or architecture:

1. Place the PDFium binary in `python/tablers/` following the naming convention.
2. Add the filename to the `all_libs` list in `scripts/build_libs.json`.
3. Update the platform logic in `scripts/pre-build.py` and `scripts/post-build.py` if the new platform requires special handling (e.g., renaming).
