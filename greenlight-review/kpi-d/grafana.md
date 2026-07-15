---
sidebar_position: 2
---

# Grafana in This Project

## Overview

[Grafana](https://grafana.com/) is a dashboard and observability platform. In this project, we use it **only as a data visualization tool**.

We use Grafana to:

- compare software performance between releases
- identify regressions or improvements
- build reusable dashboards
- create clear charts for internal reports

:::note
We do **not** use Grafana in this project for real-time monitoring, alerting, or incident response.
:::

---

## Why We Use Grafana

Grafana is commonly used for **time-based monitoring**, but it still works well for our needs because it provides:

- reusable dashboards
- flexible chart types
- side-by-side comparisons
- presentation-friendly visualizations
- direct querying of structured data

In this project, Grafana should be understood as a **visualization layer over SQL data**, not as a live observability platform.

Our use case is slightly different from Grafana’s primary design: most of our data is **not real-time**, and some of it is not naturally time-based at all. Even so, Grafana remains valuable because it gives us a practical and consistent way to present performance data across software versions.

---

## Scope of This Page

This page covers the parts of Grafana we actively use:

- dashboards
- panels (visualizations)
- SQL queries
- transformations
- version-to-version performance comparisons

This page does **not** cover:

- alerting
- notifications
- incident workflows
- permissions and access control
- full Grafana administration

:::tip
If you need to configure a new data source from scratch, Grafana’s built-in onboarding flow is usually enough. This page assumes the data source is already configured and working.
:::

---

## Project Setup

### Data Source

We use a **SQLite data source plugin** to query performance data stored in a SQLite database.

---

## Expected Data Shape

Our dashboards assume a table similar to:

```sql
system_metrics(
  plugin TEXT,
  version TEXT,
  ram_usage REAL,
  cpu_usage REAL,
  disk_usage REAL
)
```

> The exact schema can evolve, but most examples on this page assume a structure close to the one above.

---

## Working with Dashboards

In this project, nearly all of our Grafana usage happens inside **dashboards**.

A dashboard is the container where we organize queries and visualizations so we can compare software versions in a consistent and readable way.

Each dashboard usually focuses on a specific question, such as:

- how one plugin evolves across versions
- how two releases compare directly
- how multiple resource metrics behave for the same version range

:::info
This page intentionally avoids step-by-step UI instructions or version-specific screenshots, because Grafana’s interface can change between releases.
:::

---

## Dashboards

A dashboard is the starting point for almost everything we do in Grafana. When first created, it is empty.

Within a dashboard, we mainly use two capabilities:

1. creating panels (visualizations)
2. controlling the dashboard time range

---

## Panels (Visualizations)

A panel is a single visualization inside a dashboard. Each panel combines:

- a query
- optional transformations
- a visualization type (bar chart, time series, table, etc.)

Depending on the data, a panel can represent:

- a simple bar chart
- a comparison between versions
- a multi-metric line chart
- a table used for validation or reporting

### Recommended Approach

Try to keep panels focused and readable.

A common mistake is to make a single panel do too much. In practice, it is often better to:

- create **two simple panels**
- place them side by side
- compare them visually

This usually produces a clearer dashboard than forcing multiple unrelated comparisons into one complex visualization.

---

## Dashboard Time Range

Grafana dashboards have a **global time range** that affects all panels by default.

This is a core feature in traditional monitoring setups because it lets you:

- zoom into specific events
- inspect short periods before or after an incident
- explore time-based behavior in detail

In our project, however, this behavior is often **not directly useful**, because our data is usually not true time-series data.

### Panel-Level Overrides

Grafana allows you to override the time range for individual panels, but in our workflow it is usually better to:

- keep dashboards conceptually simple
- create a separate dashboard when a different comparison context is needed

This tends to be easier to maintain than mixing multiple time scopes in one dashboard.

---

## Transformations

Because Grafana is heavily optimized for time-based visualization, some of our data needs to be adapted before it can be displayed effectively.

This is where **transformations** become important.

A transformation lets Grafana reshape or reinterpret query results. In our use case, transformations are especially useful when we need to:

- convert values into a form Grafana can plot more easily
- adapt non-time data to time-oriented visualizations
- prepare version-based data for comparison charts

In practice, we sometimes use transformations to make non-temporal values behave more like time-compatible data when a panel expects it.

:::tip
If a chart feels awkward to build, check whether the problem should be solved with a transformation rather than with a more complicated SQL query.
:::

---

## SQL Queries

Since we use **SQLite** as the data source, we retrieve data through SQL queries.

Other Grafana data sources may use different query systems, but in this project the workflow is straightforward:

1. query the performance data with SQL
2. optionally apply transformations
3. choose the most suitable visualization

---

## Example Queries

Below are two common query patterns we use.

### 1. Compare performance evolution across minor versions of a plugin

This query shows multiple metrics for a specific plugin across a range of versions.  
The `threshold` column is included so it can be used to draw a threshold line in the chart if needed.

```sql
SELECT
  version,
  ram_usage,
  cpu_usage,
  disk_usage,
  40 AS threshold
FROM system_metrics
WHERE plugin = 'Scanner'
  AND version > 0
  AND version < 1;
```

### 2. Compare two major versions directly

This query is useful when you want a direct comparison between two specific releases.

```sql
SELECT
  version,
  ram_usage,
  cpu_usage,
  disk_usage
FROM system_metrics
WHERE plugin = 'Scanner'
  AND (version = 0.1 OR version = 1.0);
```

---

## Practical Usage Patterns

The most common dashboard patterns in this project are:

- **performance evolution across versions**  
  Useful for spotting trends, regressions, or gradual increases in resource usage.

- **direct version-to-version comparison**  
  Useful for release validation, especially before and after major changes.

- **multi-metric views**  
  Useful when RAM, CPU, and disk usage need to be reviewed together for the same plugin and version set.

When possible, prefer dashboards that answer **one clear question** rather than dashboards that try to summarize everything at once.

---

## Summary

In this project, Grafana is best understood as:

- a **dashboard tool**
- a **SQL-backed visualization layer**
- a **reporting aid for software performance comparisons**

We use it to:

- compare releases
- visualize performance changes
- identify regressions and improvements
- generate readable charts for internal analysis and reporting

We do **not** use it as a full observability or incident-response platform.

---
