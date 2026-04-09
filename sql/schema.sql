CREATE TABLE system_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    plugin TEXT NOT NULL,
    ram_usage REAL NOT NULL,
    cpu_usage REAL NOT NULL,
    disk_usage REAL NOT NULL,
    is_active INTEGER,
    version REAL NOT NULL
);