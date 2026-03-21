#!/usr/bin/env python3
"""Build a platform-specific wheel containing the chub binary.

Usage:
    python build_wheel.py --binary path/to/chub --target <rust-target> --version 0.1.0 --output dist/

This creates a wheel with the correct platform tag for the given Rust target triple.
"""

import argparse
import hashlib
import os
import shutil
import stat
import sys
import tempfile
import zipfile
from base64 import urlsafe_b64encode
from pathlib import Path

# Rust target triple -> Python wheel platform tag
PLATFORM_TAGS = {
    "x86_64-unknown-linux-gnu": "manylinux_2_17_x86_64.manylinux2014_x86_64",
    "aarch64-unknown-linux-gnu": "manylinux_2_17_aarch64.manylinux2014_aarch64",
    "x86_64-apple-darwin": "macosx_10_12_x86_64",
    "aarch64-apple-darwin": "macosx_11_0_arm64",
    "x86_64-pc-windows-msvc": "win_amd64",
}


def sha256_digest(path: str) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(8192), b""):
            h.update(chunk)
    return urlsafe_b64encode(h.digest()).rstrip(b"=").decode("ascii")


def file_size(path: str) -> int:
    return os.path.getsize(path)


def build_wheel(binary: str, target: str, version: str, output: str) -> str:
    platform_tag = PLATFORM_TAGS.get(target)
    if not platform_tag:
        print(f"Unknown target: {target}", file=sys.stderr)
        print(f"Supported: {', '.join(PLATFORM_TAGS)}", file=sys.stderr)
        sys.exit(1)

    wheel_name = f"chub-{version}-py3-none-{platform_tag}.whl"
    dist_info = f"chub-{version}.dist-info"

    with tempfile.TemporaryDirectory() as tmp:
        tmp = Path(tmp)

        # Copy binary into chub/bin/
        bin_dir = tmp / "chub" / "bin"
        bin_dir.mkdir(parents=True)

        binary_name = os.path.basename(binary)
        dest_binary = bin_dir / binary_name
        shutil.copy2(binary, dest_binary)
        os.chmod(dest_binary, os.stat(dest_binary).st_mode | stat.S_IEXEC)

        # Copy Python source files
        src_dir = Path(__file__).parent / "chub"
        for py_file in ["__init__.py", "__main__.py"]:
            shutil.copy2(src_dir / py_file, tmp / "chub" / py_file)

        # Create dist-info
        info_dir = tmp / dist_info
        info_dir.mkdir()

        # METADATA
        (info_dir / "METADATA").write_text(
            f"Metadata-Version: 2.1\n"
            f"Name: chub\n"
            f"Version: {version}\n"
            f"Summary: Chub: high-performance curated docs for AI coding agents\n"
            f"Home-page: https://github.com/vietanhdev/chub\n"
            f"License: MIT\n"
            f"Requires-Python: >=3.8\n"
            f"Author-email: Viet-Anh Nguyen <vietanh.dev@gmail.com>\n"
        )

        # WHEEL
        (info_dir / "WHEEL").write_text(
            f"Wheel-Version: 1.0\n"
            f"Generator: chub-build-wheel\n"
            f"Root-Is-Purelib: false\n"
            f"Tag: py3-none-{platform_tag}\n"
        )

        # entry_points.txt
        (info_dir / "entry_points.txt").write_text(
            "[console_scripts]\nchub = chub:main\n"
        )

        # Collect all files for RECORD
        records = []
        all_files = []
        for root, _dirs, files in os.walk(tmp):
            for f in files:
                full = os.path.join(root, f)
                rel = os.path.relpath(full, tmp).replace("\\", "/")
                all_files.append((full, rel))
                h = sha256_digest(full)
                s = file_size(full)
                records.append(f"{rel},sha256={h},{s}")

        record_path = info_dir / "RECORD"
        records.append(f"{dist_info}/RECORD,,")
        record_path.write_text("\n".join(records) + "\n")

        # Build the wheel zip
        os.makedirs(output, exist_ok=True)
        wheel_path = os.path.join(output, wheel_name)
        with zipfile.ZipFile(wheel_path, "w", zipfile.ZIP_DEFLATED) as whl:
            for full, rel in all_files:
                whl.write(full, rel)
            whl.write(record_path, f"{dist_info}/RECORD")

    print(f"Built {wheel_path}")
    return wheel_path


def main():
    parser = argparse.ArgumentParser(description="Build chub platform wheel")
    parser.add_argument("--binary", required=True, help="Path to compiled chub binary")
    parser.add_argument("--target", required=True, help="Rust target triple")
    parser.add_argument("--version", required=True, help="Package version")
    parser.add_argument("--output", default="dist", help="Output directory")
    args = parser.parse_args()

    build_wheel(args.binary, args.target, args.version, args.output)


if __name__ == "__main__":
    main()
