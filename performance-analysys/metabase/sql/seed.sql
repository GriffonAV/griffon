INSERT INTO scan_metrics (target_type, hit, total, ts) VALUES
('ELF', 300, 1000, '2026-01-01'),
('PE', 500, 1500, '2026-01-01'),
('PDF', 200, 800, '2026-01-01');

INSERT INTO scan_metrics (target_type, hit, total, ts) VALUES
('ELF', 400, 1000, '2026-01-02'),
('PE', 300, 1500, '2026-01-02'),
('PDF', 800, 800, '2026-01-02');


INSERT INTO system_metrics (ram_usage, cpu_usage, disk_usage, is_active, ts) VALUES
(100, 30, 20, 1, '2026-01-01'),
(59, 45, 25, 1, '2026-01-01'),
(44, 50, 13, 1, '2026-01-01'),
(10, 2, 8, 0, '2026-01-01'),
(10, 1, 9, 0, '2026-01-01'),
(10, 0.01, 15, 0, '2026-01-01');

INSERT INTO system_metrics (ram_usage, cpu_usage, disk_usage, is_active, ts) VALUES
(56, 40, 20, 1, '2026-01-02'),
(5, 41, 25, 1, '2026-01-02'),
(42, 90, 13, 1, '2026-01-02'),
(60, 15, 1, 0, '2026-01-02'),
(5, 0.21, 85, 0, '2026-01-02'),
(11, 4.01, 14, 0, '2026-01-02');