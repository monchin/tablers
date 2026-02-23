# Installation

## Requirements

Before installing Tablers, ensure your system meets the following requirements:

- **Python**: >= 3.10
- **Operating System**:
    - Windows (x64)
    - Linux (x64), glibc >= 2.28 (manylinux_2_28)
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
