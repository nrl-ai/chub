"""Chub: high-performance curated docs for AI coding agents."""

__version__ = "0.1.5"

import os
import platform
import subprocess
import sys


def _get_binary_path() -> str:
    """Resolve the path to the native chub binary bundled in this wheel."""
    pkg_dir = os.path.dirname(os.path.abspath(__file__))
    exe = "chub.exe" if platform.system() == "Windows" else "chub"
    return os.path.join(pkg_dir, "bin", exe)


def main() -> None:
    binary = _get_binary_path()
    if not os.path.isfile(binary):
        print(
            f"chub binary not found at {binary}\n"
            "This likely means the wheel was not built for your platform.\n"
            "Please install the correct platform wheel or build from source.",
            file=sys.stderr,
        )
        sys.exit(1)
    try:
        result = subprocess.run([binary, *sys.argv[1:]])
        sys.exit(result.returncode)
    except KeyboardInterrupt:
        sys.exit(130)
