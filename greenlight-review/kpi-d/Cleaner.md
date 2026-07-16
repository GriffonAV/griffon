---
sidebar_position: 1
---

# Cleaner module

This module is meant to free space on your computer by checking for unused configurations files, temp files and many other type of files that could sit unused on your computer.


## performance data export

example of the data that can be exported from the cleaner module.

```json
{
  "scenario": "basic_clean",
  "version": "v0.0.9",
  "mode": "aggressive",
  "plugin": "griffon_cleaner",
  "timestamp": "2026-07-01T12:50:05+00:00",
  "bytes_freed_total": 15812600355,
  "files_scanned_total": 103018,
  "files_cleaned_total": 5044,
  "run_duration_seconds": 1.668,
  "bytes_freed_per_second": 9479976232.014389,
  "errors_by_type": {
    "permission_denied": 25
  }
}
```

## performance metrics

The metrics used to measure the efficency of this module are the following: 
- bytes analysed per second
- bytes freed total
- time of the scan
- errors encountered

### bytes analysed per second

The field of cleaning up space on a computer is not something critical that would require speed at any cost, unlike some other field such as network request proccessing or anything related to security, but it is one of the thing that users look at the most.

Because of that the speed at which this module can handle data becomes something really important for user satisfaction and by extension, user retention.

The metric in itself is self explanatory, it's the ammount of data the module scan in a second. This metric is obtained by dividing the total ammount of bytes scanned divided by the time it took for the scan to complete.

### bytes freed total

The quantity of space freed is relative to a constant based on the scenario used.
It is usefull to measure if the module cleaned all the files that were introduced in the test.

### time of the scan

The time of the scan is another speed metrics that is usefull to see the limit of the cleaner model and warn the users when they try to run a clean that would take a long time. It is essentialy data meant to be used to fine-tune 

### errors encountered

This metrics is used to filter outlier that would be caused by a string of multiples errors that could slow down the overall scan.