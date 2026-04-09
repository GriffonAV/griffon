-- example seed data for system_metrics table

INSERT INTO system_metrics (plugin, ram_usage, cpu_usage, disk_usage, is_active, version) VALUES
-- Initial versions
('Cleaner', 12, 15.5, 20, 1, 0.1),
('Scanner', 32, 18.2, 25, 1, 0.1),

-- Version 0.2
('Cleaner', 11, 14.8, 19, 1, 0.2),
('Scanner', 34, 19.5, 27, 1, 0.2),

-- Version 0.3
('Cleaner', 10, 13.6, 18, 1, 0.3),
('Scanner', 37, 21.0, 30, 1, 0.3),

-- Version 0.4
('Cleaner', 9, 12.9, 17, 1, 0.4),
('Scanner', 40, 22.7, 33, 1, 0.4),

-- Version 0.5
('Cleaner', 8, 11.7, 15, 1, 0.5),
('Scanner', 44, 24.8, 36, 1, 0.5),

-- Version 1.0
('Cleaner', 7, 10.5, 14, 1, 1.0),
('Scanner', 48, 27.2, 40, 1, 1.0);
