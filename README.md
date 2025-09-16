# Relay Curve Estimator

A Python-based GUI application for estimating relay curves, built with PySide6.

## Features

- Intuitive graphical user interface for relay curve estimation
- Built with PySide6 for cross-platform compatibility
- Easy-to-use executable for Windows

## Requirements

- Python 3.12 or higher
- PySide6 6.9.2 or higher

## Installation

This project uses [uv](https://github.com/astral-sh/uv) for dependency management.

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd relay-curve-estimator
   ```

2. Install dependencies using uv:
   ```bash
   uv sync
   ```

3. Run the application:
   ```bash
   uv run python curve_estimator_gui.py
   ```

## Usage

Launch the GUI application to input parameters and estimate relay curves. The application provides a user-friendly interface for configuring and visualizing relay curve estimations.

## Building (cx_Freeze)

This project uses cx_Freeze to create a Windows executable and MSI installer.

- Build the executable:

```
uv run python .\setup.py build
```

Output: `build/CurveEstimator/CurveEstimator.exe`

- Build the MSI installer (optional):

```
uv run python .\setup.py bdist_msi
```

Output: `dist/RelayCurveEstimator-0.1.0-win64.msi`

Notes:
- The installer creates a Start Menu shortcut named "Relay Curve Estimator".
- The build excludes unused Qt modules to keep size smaller; adjust `PYSIDE_EXCLUDES` in `setup.py` if you add features.

## Project Structure

- `curve_estimator_gui.py`: Main GUI application
- `curve_estimator.py`: Core estimation logic
- `pyproject.toml`: Project configuration and dependencies

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
