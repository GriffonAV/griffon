CREATE TABLE scan_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    target_type TEXT NOT NULL,
    hit INTEGER NOT NULL,
    total INTEGER NOT NULL,
    ts TEXT NOT NULL
);

CREATE TABLE system_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ram_usage REAL NOT NULL,
    cpu_usage REAL NOT NULL,
    disk_usage REAL NOT NULL,
    is_active INTEGER,
    ts TEXT NOT NULL
);