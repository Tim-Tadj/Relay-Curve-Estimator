from cx_Freeze import setup, Executable
import sys

# -----------------------------
# Application metadata
# -----------------------------
APP_NAME = "Relay Curve Estimator"
APP_VERSION = "0.1.0"  # Keep in sync with pyproject.toml
APP_DESCRIPTION = "Protection Curve Estimator"
TARGET_EXE = "CurveEstimator.exe"
BUILD_DIR = "build/CurveEstimator"

# MSI/Installer metadata (set a stable GUID to enable upgrades)
UPGRADE_CODE = "{8CC6EA3A-9A75-47A7-BBE6-4D5A4E8B8C0F}"
TARGET_DIR = r"[ProgramFilesFolder]\\Relay Curve Estimator"
MSI_TARGET_NAME = "RelayCurveEstimator"

# -----------------------------
# Dependency inclusion/exclusion
# -----------------------------
PACKAGES = [
    "os",
    "sys",
    "time",
    "re",
    "warnings",
    "json",
    "logging",
    "collections",
    "importlib",
    "ctypes",
    "PySide6",
    "PySide6.QtWidgets",
    "PySide6.QtCore",
    "PySide6.QtGui",
]

INCLUDES = [
    "PySide6.QtWidgets",
    "PySide6.QtCore",
    "PySide6.QtGui",
]

# Add data files here if needed, e.g. ("assets/icon.png", "icon.png")
INCLUDE_FILES = []

BIN_EXCLUDES = [
    # Not used in this app; save size by excluding heavy Qt WebEngine
    "Qt6WebEngineCore.dll",
    "Qt6WebEngine.dll",
    "Qt6WebEngineWidgets.dll",
    "QtPdf.dll",
    "QtPdfQuick.dll",
]

PYSIDE_EXCLUDES = [
    "PySide6.Qt3DAnimation",
    "PySide6.Qt3DCore",
    "PySide6.Qt3DExtras",
    "PySide6.Qt3DInput",
    "PySide6.Qt3DLogic",
    "PySide6.Qt3DRender",
    "PySide6.QtCharts",
    "PySide6.QtConcurrent",
    "PySide6.QtDataVisualization",
    "PySide6.QtGraphs",
    "PySide6.QtMultimedia",
    "PySide6.QtMultimediaWidgets",
    "PySide6.QtNetworkAuth",
    "PySide6.QtOpenGL",
    "PySide6.QtOpenGLWidgets",
    "PySide6.QtPositioning",
    "PySide6.QtQml",
    "PySide6.QtQmlModels",
    "PySide6.QtQuick",
    "PySide6.QtQuick3D",
    "PySide6.QtQuickControls2",
    "PySide6.QtQuickWidgets",
    "PySide6.QtRemoteObjects",
    "PySide6.QtSensors",
    "PySide6.QtSerialPort",
    "PySide6.QtStateMachine",
    "PySide6.QtTextToSpeech",
    "PySide6.QtVirtualKeyboard",
    "PySide6.QtWebChannel",
    "PySide6.QtWebEngine",
    "PySide6.QtWebEngineCore",
    "PySide6.QtWebEngineQuick",
    "PySide6.QtWebEngineWidgets",
    "PySide6.QtWebSockets",
    "PySide6.QtWebView",
    "PySide6.QtXml",
    "PySide6.QtXmlPatterns",
    "PySide6.QtDesigner",
    "PySide6.QtGraphsWidgets",
    "PySide6.QtHelp",
    "PySide6.QtNfc",
    "PySide6.QtHttpServer",
    "PySide6.QtLocation",
    "PySide6.QtPdf",
    "PySide6.QtPdfWidgets",
    "PySide6.QtPositioningQuick",
    "PySide6.QtQuickShapes",
    "PySide6.QtScxml",
    "PySide6.QtSerialBus",
    "PySide6.QtShaderTools",
    "PySide6.QtSvg",
    "PySide6.QtSvgWidgets",
    "PySide6.QtTest",
    "PySide6.QtUiTools",
]

THIRD_PARTY_EXCLUDES = [
    "importlib_metadata",
    "setuptools",
    "wheel",
    "zipp",
]

build_exe_options = {
    "packages": PACKAGES,
    "includes": INCLUDES,
    "include_files": INCLUDE_FILES,
    "include_msvcr": True,
    "excludes": [
        "tkinter",
        "unittest",
        "pydoc",
        "doctest",
        "pytest",
        "email",
        "http",
        "xml",
        "xmlrpc",
        "asyncio",
        *PYSIDE_EXCLUDES,
        *THIRD_PARTY_EXCLUDES,
    ],
    "build_exe": BUILD_DIR,
    "bin_excludes": BIN_EXCLUDES,
    "optimize": 2,
}

# Start as a GUI app on Windows
base = "Win32GUI" if sys.platform == "win32" else None

# Start menu shortcut
shortcut_table = [
    (
        "RelayCurveEstimatorShortcut",  # Shortcut
        "ProgramMenuFolder",           # Directory_
        APP_NAME,                       # Name
        "TARGETDIR",                   # Component_
        f"[TARGETDIR]{TARGET_EXE}",    # Target
        None,                           # Arguments
        f"Launch {APP_NAME}",          # Description
        None,                           # Hotkey
        None,                           # Icon
        None,                           # IconIndex
        None,                           # ShowCmd
        "TARGETDIR",                   # WkDir
    ),
]

bdist_msi_options = {
    "upgrade_code": UPGRADE_CODE,
    "add_to_path": False,
    "all_users": False,
    "initial_target_dir": TARGET_DIR,
    "target_name": MSI_TARGET_NAME,
    "data": {"Shortcut": shortcut_table},
}

executables = [
    Executable(
        "curve_estimator_gui.py",
        base=base,
        target_name=TARGET_EXE,
    )
]

setup(
    name=APP_NAME,
    version=APP_VERSION,
    description=APP_DESCRIPTION,
    options={"build_exe": build_exe_options, "bdist_msi": bdist_msi_options},
    executables=executables,
)
