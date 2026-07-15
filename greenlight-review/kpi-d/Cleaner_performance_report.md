# Performance Report – Cleaner Module

## Version Comparison

**Baseline Version:** 0.0.9
**Test Version:** 0.1.0

---

# Objective

This performance report evaluates the impact of the changes introduced in version **0.1.0** of the Cleaner Module compared to **0.0.9**.

The report focuses on three main objectives:

1. Verify that the optimized folder navigation introduced in version 0.1.0 improves scan performance.
2. Evaluate the performance impact of expanding the list of files processed by the **Aggressive Scan** mode compared to the **Safe Scan** mode.
3. Assess whether the observed performance trade-offs remain acceptable considering the intended functionality of each scan mode.

---

# Test Environment

| Parameter        | Value                |
| ---------------- | -------------------- |
| Previous Version | 0.0.9                |
| Current Version  | 0.1.0                |
| Operating System | generic/ubuntu2204   |
| Hardware         | memory(4096) cpus(2) |
| Number of Runs   | 10                   |

---

# Results

## 1. Folder Navigation Optimization

### Overview

Version **0.1.0** introduces an optimized folder navigation algorithm intended to reduce traversal overhead during directory scanning.

The objective of this test is to determine whether the optimization results in faster scan execution while maintaining identical scan results.

### Measurements

| Metric                         | Version 0.0.9 | Version 0.1.0 | Difference |
| -------------------------      | ------------- | ------------- | ---------- |
| Average Safe Scan Speed        |  9.10 g/s      |  11.70 g/s    | 2.6 g/s    |
| Average Aggressive Scan Speed   |  9.09 g/s      |  11.00 g/s    | 1.91 g/s   |

g/s = gigabytes per second

### Performance Dashboard

![grafana dashboard](./cleaner_modes_comparison.png)

---

### Analysis

The collected data indicates that version **0.1.0** performs directory traversal more efficiently than version **0.0.9**.

The optimized folder navigation reduces the overall scanning time, resulting in a measurable increase in scan throughput. This confirms that the implementation successfully improves the performance of the scanning phase without affecting functionality.

---

## 2. Aggressive Scan Performance

### Overview

Version **0.1.0** expands the list of file types processed by the **Aggressive Scan** mode. Unlike **Safe Scan**, which targets only well-established removable files, Aggressive Scan now evaluates additional file categories to provide a more exhaustive cleaning operation.

The purpose of this analysis is to quantify the performance impact of this functional expansion.

### Measurements

| Metric                      | Safe Scan | Aggressive Scan |
| --------------------------- | --------- | --------------- |
| Files Examined              | 103 018    | 108 793          |
| Files Eligible for Cleaning | 5 143      | 7 093            |
| Scan Speed                  | 11.7 g/s  | 11.0 g/s        |

g/s = gigabytes per second

### Analysis

The benchmark data shows that the **Aggressive Scan** mode executes more slowly than the **Safe Scan** compared to the previous version.

This behavior is expected because the module now evaluates a larger number of file types during the scan. The increased workload naturally results in longer processing times, even though the scanning algorithm itself remains efficient.

The observed decrease in scan speed is therefore a consequence of expanded functionality rather than a performance regression.

---

## 3. Performance Trade-off Assessment

### Overview

The final objective is to determine whether the additional execution time introduced by the Aggressive Scan remains acceptable given the benefits it provides.

### Analysis

Although Aggressive Scan requires additional processing time, the increase is proportional to the broader validation performed on each scanned item.

The additional file categories improve the module's ability to distinguish between removable files and files that should be preserved. This added verification reduces the likelihood of deleting files that may still be important to the user.

As a result, the decrease in scan speed represents a deliberate design trade-off in favor of improved cleaning reliability and user safety.

---

# Conclusion

The performance evaluation of version **0.1.0** demonstrates that the Cleaner Module successfully achieves its primary performance objective.

The optimized folder navigation produces a measurable increase in overall scan speed, confirming the effectiveness of the new traversal implementation.

The expanded Aggressive Scan introduces additional processing overhead, leading to longer scan times compared to Safe Scan. However, this slowdown is expected due to the increased number of file types evaluated during the cleaning process and does not indicate a regression in the scanning engine.

Overall, version **0.1.0** delivers improved general scanning performance while extending the functionality of the Aggressive Scan mode.

---

# Follow-up

The results of this evaluation suggest that the folder navigation optimization is ready for production use and should remain the default implementation moving forward.

While the Aggressive Scan mode exhibits lower scan speed, the additional processing provides more comprehensive file validation and reduces the risk of removing files that may still be valuable to the user. Given the purpose of this mode, the current performance trade-off is considered acceptable.

Future work should focus on identifying opportunities to optimize the additional validation steps introduced in Aggressive Scan without reducing the quality of the cleaning process. Any future performance improvements should preserve the current emphasis on safe and reliable file handling.
