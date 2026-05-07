#!/usr/bin/env python3
"""
fetch_eicar.py — downloads the 4 official EICAR test files

Safe to run anywhere: EICAR is a harmless AV test standard, not real malware.
Output goes to tests/fixtures/files/eicar/ by default.

Usage:
    python3 tests/fixtures/fetch_eicar.py
    python3 tests/fixtures/fetch_eicar.py --out /tmp/eicar
"""

import argparse
import urllib.request
from pathlib import Path

EICAR_FILES = [
    "https://secure.eicar.org/eicar.com",
    "https://secure.eicar.org/eicar.com.txt",
    "https://secure.eicar.org/eicar_com.zip",
    "https://secure.eicar.org/eicar_com2.zip",
]

def fetch(out_dir: Path):
    out_dir.mkdir(parents=True, exist_ok=True)
    for url in EICAR_FILES:
        name = url.split("/")[-1]
        dest = out_dir / name
        if dest.exists():
            print(f"  skip     {name} (already exists)")
            continue
        print(f"  fetching {name} ...", end=" ", flush=True)
        urllib.request.urlretrieve(url, dest)
        print(f"{dest.stat().st_size} bytes")
    print(f"\nDone → {out_dir}")

if __name__ == "__main__":
    p = argparse.ArgumentParser(description="Download EICAR test files")
    p.add_argument("--out", default="tests/fixtures/files/eicar", help="Output directory")
    args = p.parse_args()
    fetch(Path(args.out))