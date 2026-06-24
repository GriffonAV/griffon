#!/usr/bin/env python3
"""Parse a specific Griffon cleaner JSON and store structured rows for Grafana.

This script targets the JSON layout found in `grafana/cleaner_source/test_multi.json`.
It creates three tables:
- `runs`: top-level metadata for the run (run_id, plugin, generated_at, totals)
- `modules`: per-module metrics that are easy to query/visualize in Grafana
- `raw_json`: optional raw JSON blob for reference

Usage:
  python3 json_to_sqlite.py --input grafana/cleaner_source/test_multi.json

Defaults: DB = `sql/griffon.db` (created if missing). Use `--db` to change.
"""

from __future__ import annotations

import argparse
import datetime
import json
import os
import sqlite3
import sys
from typing import Any, Dict, Optional


SCHEMA_RUNS = """
CREATE TABLE IF NOT EXISTS runs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  run_id TEXT,
  plugin_name TEXT,
  plugin_version TEXT,
  generated_at TEXT,
  total_files_touched INTEGER,
  total_bytes_freed INTEGER,
  total_warnings INTEGER,
  total_errors INTEGER,
  total_permission_denied INTEGER,
  total_duration_ms INTEGER,
  inserted_at TEXT
)
"""

SCHEMA_MODULES = """
CREATE TABLE IF NOT EXISTS modules (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  run_fk INTEGER REFERENCES runs(id),
  module_id TEXT,
  duration_ms INTEGER,
  files_touched INTEGER,
  bytes_freed INTEGER,
  warnings_count INTEGER,
  errors_count INTEGER,
  permission_denied INTEGER,
  candidate_files_count INTEGER,
  deleted_files_count INTEGER,
  skipped_files_count INTEGER,
  missing_paths_count INTEGER,
  existing_paths_count INTEGER,
  delete_success_rate REAL,
  warning_rate REAL,
  avg_bytes_per_file REAL,
  bytes_per_second REAL
)
"""

SCHEMA_RAW = """
CREATE TABLE IF NOT EXISTS raw_json (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  run_fk INTEGER REFERENCES runs(id),
  content TEXT,
  inserted_at TEXT
)
"""


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description="Insert Griffon cleaner JSON into SQLite for Grafana")
    p.add_argument("--input", "-i", required=True, help="Path to JSON file, or '-' to read stdin")
    p.add_argument("--db", "-d", default=os.path.join("sql", "griffon.db"), help="SQLite DB path")
    p.add_argument("--raw", action="store_true", help="Also store raw JSON blob")
    p.add_argument("--replace", action="store_true", help="Drop and recreate tables before inserting")
    return p.parse_args()


def read_input(path: str) -> str:
    if path == "-":
        return sys.stdin.read()
    with open(path, "r", encoding="utf-8") as f:
        return f.read()


def ensure_schema(conn: sqlite3.Connection, replace: bool = False) -> None:
    cur = conn.cursor()
    if replace:
        cur.execute("DROP TABLE IF EXISTS raw_json")
        cur.execute("DROP TABLE IF EXISTS modules")
        cur.execute("DROP TABLE IF EXISTS runs")
    cur.executescript(SCHEMA_RUNS)
    cur.executescript(SCHEMA_MODULES)
    cur.executescript(SCHEMA_RAW)
    conn.commit()


def safe_int(v: Any) -> Optional[int]:
    try:
        if v is None:
            return None
        return int(v)
    except Exception:
        return None


def safe_float(v: Any) -> Optional[float]:
    try:
        if v is None:
            return None
        return float(v)
    except Exception:
        return None


def insert_run(conn: sqlite3.Connection, data: Dict[str, Any]) -> int:
    cur = conn.cursor()
    report = data.get("report", {})
    run_id = data.get("run_id") or data.get("report", {}).get("run_id")
    plugin_name = data.get("plugin_name")
    plugin_version = data.get("plugin_version")
    generated_at = data.get("generated_at")

    totals = report if isinstance(report, dict) else {}

    cur.execute(
        """
        INSERT INTO runs (run_id, plugin_name, plugin_version, generated_at,
                          total_files_touched, total_bytes_freed, total_warnings,
                          total_errors, total_permission_denied, total_duration_ms, inserted_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            run_id,
            plugin_name,
            plugin_version,
            generated_at,
            safe_int(totals.get("total_files_touched") or totals.get("total_files_touched")),
            safe_int(totals.get("total_bytes_freed") or totals.get("total_bytes_freed")),
            safe_int(totals.get("total_warnings") or totals.get("total_warnings")),
            safe_int(totals.get("total_errors") or totals.get("total_errors")),
            safe_int(totals.get("total_permission_denied") or totals.get("total_permission_denied")),
            safe_int(totals.get("total_duration_ms") or totals.get("total_duration_ms")),
            datetime.datetime.utcnow().isoformat(),
        ),
    )
    conn.commit()
    return cur.lastrowid


def insert_module(conn: sqlite3.Connection, run_fk: int, module_id: str, obj: Dict[str, Any]) -> int:
    cur = conn.cursor()
    cur.execute(
        """
        INSERT INTO modules (
            run_fk, module_id, duration_ms, files_touched, bytes_freed,
            warnings_count, errors_count, permission_denied, candidate_files_count,
            deleted_files_count, skipped_files_count, missing_paths_count, existing_paths_count,
            delete_success_rate, warning_rate, avg_bytes_per_file, bytes_per_second
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """,
        (
            run_fk,
            module_id,
            safe_int(obj.get("duration_ms")),
            safe_int(obj.get("files_touched")),
            safe_int(obj.get("bytes_freed")),
            safe_int(obj.get("warnings_count") or obj.get("warnings")),
            safe_int(obj.get("errors_count") or obj.get("errors")),
            safe_int(obj.get("permission_denied")),
            safe_int(obj.get("candidate_files_count")),
            safe_int(obj.get("deleted_files_count")),
            safe_int(obj.get("skipped_files_count") or obj.get("skipped_files_count")),
            safe_int(obj.get("missing_paths_count")),
            safe_int(obj.get("existing_paths_count")),
            safe_float(obj.get("delete_success_rate")),
            safe_float(obj.get("warning_rate")),
            safe_float(obj.get("avg_bytes_per_file")),
            safe_float(obj.get("bytes_per_second")),
        ),
    )
    conn.commit()
    return cur.lastrowid


def store_raw(conn: sqlite3.Connection, run_fk: int, raw: str) -> int:
    cur = conn.cursor()
    cur.execute(
        "INSERT INTO raw_json (run_fk, content, inserted_at) VALUES (?, ?, ?)",
        (run_fk, raw, datetime.datetime.utcnow().isoformat()),
    )
    conn.commit()
    return cur.lastrowid


def main() -> None:
    args = parse_args()

    raw = read_input(args.input).strip()
    if not raw:
        print("No input JSON provided", file=sys.stderr)
        sys.exit(2)

    try:
        data = json.loads(raw)
    except json.JSONDecodeError as exc:
        print("Invalid JSON:", exc, file=sys.stderr)
        sys.exit(3)

    db_dir = os.path.dirname(args.db)
    if db_dir and not os.path.exists(db_dir):
        os.makedirs(db_dir, exist_ok=True)

    conn = sqlite3.connect(args.db)
    try:
        ensure_schema(conn, replace=args.replace)

        run_fk = insert_run(conn, data)

        # Insert modules from report.per_module if present
        report = data.get("report") or {}
        per_module = report.get("per_module") or {}
        if isinstance(per_module, dict) and per_module:
            for mod_id, mod_obj in per_module.items():
                if isinstance(mod_obj, dict):
                    insert_module(conn, run_fk, mod_id, mod_obj)

        # Also insert summary modules from analysis.* arrays (if any)
        analysis = data.get("analysis") or {}
        for key in ("modules_by_bytes_freed", "modules_by_duration", "modules_by_warnings"):
            arr = analysis.get(key) or []
            if isinstance(arr, list):
                for obj in arr:
                    mod_id = obj.get("module_id")
                    if mod_id:
                        insert_module(conn, run_fk, mod_id, obj)

        if args.raw:
            store_raw(conn, run_fk, raw)

    finally:
        conn.close()

    print(f"Inserted run id {run_fk} into DB: {args.db}")


if __name__ == "__main__":
    main()
