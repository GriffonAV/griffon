INSERT INTO scan_metrics (target_type, hit, total, ts) VALUES
('ELF', 300, 1000, '2026-01-01'),
('PE', 500, 1500, '2026-01-01'),
('PDF', 200, 800, '2026-01-01');

INSERT INTO scan_metrics (target_type, hit, total, ts) VALUES
('ELF', 400, 1000, '2026-01-02'),
('PE', 300, 1500, '2026-01-02'),
('PDF', 800, 800, '2026-01-02');


INSERT INTO disk_usage (value, is_active, time) VALUES
(20, 1, 1771236709),
(25, 1, 1771240309),
(13, 1, 1771243909),
(10, 0, 1771247509),
(9, 0, 1771251109),
(7.89, 1, 1771254709);


INSERT INTO cpu_usage (value, is_active, time) VALUES
(15.5, 1, 1771236709),
(18.2, 1, 1771240309),
(12.3, 1, 1771243909),
(10.0, 0, 1771247509),
(8.5, 0, 1771251109),
(6.7, 1, 1771254709);

INSERT INTO ram_usage (value, is_active, time) VALUES
(10.5, 1, 1771236709),
(12.3, 1, 1771240309),
(8.7, 1, 1771243909),
(6.0, 0, 1771247509),
(5.5, 0, 1771251109),
(4.2, 1, 1771254709);