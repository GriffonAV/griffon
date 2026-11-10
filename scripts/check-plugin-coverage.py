#!/usr/bin/env python3
"""Enforce line coverage separately for each plugin, excluding test-only files."""
import json
import sys
from pathlib import Path

PLUGINS = ("griffon_cleaner", "griffon_nmap")
THRESHOLD = 70.0


def main():
    report = json.loads(Path(sys.argv[1]).read_text())
    files = {}
    for data in report["data"]:
        for item in data["files"]:
            files[item["filename"]] = item
    failed = False
    print("| Plugin | Covered lines | Line coverage | Required |")
    print("| --- | ---: | ---: | ---: |")
    for plugin in PLUGINS:
        selected = [
            item for filename, item in files.items()
            if f"/plugins/{plugin}/src/" in filename.replace("\\", "/")
            and not Path(filename).stem.endswith("tests")
        ]
        count = sum(item["summary"]["lines"]["count"] for item in selected)
        covered = sum(item["summary"]["lines"]["covered"] for item in selected)
        percent = 100.0 * covered / count if count else 0.0
        failed |= count == 0 or percent < THRESHOLD
        print(f"| {plugin} | {covered}/{count} | {percent:.2f}% | {THRESHOLD:.0f}% |")
    return int(failed)


if __name__ == "__main__":
    raise SystemExit(main())
