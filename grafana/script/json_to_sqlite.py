#!/usr/bin/env python3

import json
import sqlite3
import sys
from pathlib import Path


def create_table(cursor):
    cursor.execute("""
    CREATE TABLE IF NOT EXISTS cleaner_runs (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        scenario TEXT,
        version TEXT,
        mode TEXT,
        plugin TEXT,
        timestamp TEXT,
        bytes_freed_total INTEGER,
        files_scanned_total INTEGER,
        files_cleaned_total INTEGER,
        run_duration_seconds REAL,
        bytes_freed_per_second REAL,
        errors_by_type TEXT
    )
    """)


def insert_json(cursor, data):
    cursor.execute("""
    INSERT INTO cleaner_runs (
        scenario,
        version,
        mode,
        plugin,
        timestamp,
        bytes_freed_total,
        files_scanned_total,
        files_cleaned_total,
        run_duration_seconds,
        bytes_freed_per_second,
        errors_by_type
    )
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    """, (
        data.get("scenario"),
        data.get("version"),
        data.get("mode"),
        data.get("plugin"),
        data.get("timestamp"),
        data.get("bytes_freed_total"),
        data.get("files_scanned_total"),
        data.get("files_cleaned_total"),
        data.get("run_duration_seconds"),
        data.get("bytes_freed_per_second"),
        json.dumps(data.get("errors_by_type", {}))
    ))


def main():
    if len(sys.argv) != 3:
        print(f"Usage: {Path(sys.argv[0]).name} <json_folder> <database.db>")
        sys.exit(1)

    json_folder = Path(sys.argv[1])
    db_file = sys.argv[2]

    if not json_folder.exists() or not json_folder.is_dir():
        print(f"Error: '{json_folder}' is not a valid directory.")
        sys.exit(1)

    json_files = sorted(json_folder.glob("*.json"))

    if not json_files:
        print("No JSON files found.")
        return

    conn = sqlite3.connect(db_file)
    cursor = conn.cursor()

    create_table(cursor)

    inserted = 0
    skipped = 0

    for json_file in json_files:
        try:
            with open(json_file, "r", encoding="utf-8") as f:
                data = json.load(f)

            insert_json(cursor, data)
            inserted += 1
            print(f"Inserted: {json_file.name}")

        except json.JSONDecodeError:
            skipped += 1
            print(f"Skipped (invalid JSON): {json_file.name}")

        except Exception as e:
            skipped += 1
            print(f"Skipped ({json_file.name}): {e}")

    conn.commit()
    conn.close()

    print("\nDone.")
    print(f"Files found : {len(json_files)}")
    print(f"Inserted    : {inserted}")
    print(f"Skipped     : {skipped}")


if __name__ == "__main__":
    main()