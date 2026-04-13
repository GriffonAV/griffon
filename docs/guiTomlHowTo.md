# Griffon UI Manifest Specification

**Version:** Draft 1.2
**Status:** Working Draft
**Purpose:** Define declarative user interfaces for Griffon plugins using TOML.

---

# 1. Introduction

The Griffon UI Manifest Specification defines a declarative format for describing plugin user interfaces using TOML.

A Griffon UI manifest describes:

* plugin metadata
* tabs
* sections
* UI elements contained within sections

The manifest is intentionally **presentation-focused**. It describes **what should be rendered**, not how application logic is executed.

Interactive behavior, state mutation, and relationships between UI elements are expected to be defined in a **separate logic or binding configuration**.

---

# 2. Design Principles

The Griffon UI system follows these principles:

* **Declarative**: manifests describe UI structure, not imperative behavior
* **Modular**: components are generic and reusable across plugin types
* **Separation of concerns**: UI structure is distinct from state and logic
* **Extensible**: new component types can be added without changing the overall structure
* **Predictable**: component fields use consistent naming and semantics

---

# 3. Manifest Structure

A Griffon UI manifest is composed of three major parts:

1. **Plugin metadata**
2. **Section definitions**
3. **Section contents**

### Top-level structure

```toml id="8b1u7s"
[plugin]
# plugin metadata

[[ui.sections]]
# section definition

[[ui.sections.contents]]
# UI element definition
```

---

# 4. Separation of UI and Logic

The UI manifest is strictly responsible for **presentation**.

It defines:

* what elements exist
* where they appear
* how they are labeled
* which semantic actions they emit

It does **not** define:

* business logic
* state transitions
* computed values
* cross-component data flow
* event handler implementation

These concerns are handled by a **separate logic/binding configuration**.

This separation allows:

* reusable UI manifests
* testable interaction systems
* independent rendering and execution layers
* cleaner plugin architecture

---

# 5. First Example: Button Counter

The following example demonstrates a realistic and practical UI: a simple button counter.

This interface renders:

* a title
* a grouped layout using rows and columns
* a card-like display area
* a text element that displays the current count
* an increment button
* a reset button

The buttons emit semantic actions.
The value updates are expected to be handled by a separate binding system.

---

## 5.1 UI Manifest Example

```toml id="q3rj1t"
# Griffon Plugin Manifest: Counter Demo
[plugin]
name = "Counter Demo"
id = "org.griffon.counterdemo"
version = "1.0.0"
author = "Raphael"
description = "Demonstrates a simple counter interface"
tabs = ["main"]

[[ui.sections]]
id = "counter_main"
tab = "main"

[[ui.sections.contents]]
type = "text"
id = "counter_title"
name = "Button Counter"
variant = "title"

[[ui.sections.contents]]
type = "group"
id = "counter_group"
name = "Counter Controls"

[[ui.sections.contents.children]]
type = "card"
id = "counter_card"
name = "Current Count"
description = "Press the button to increase the counter"

[[ui.sections.contents.children]]
type = "column"
id = "counter_column"
gap = "md"

[[ui.sections.contents.children.children]]
type = "text"
id = "counter_value"
name = "0"
variant = "title"
tone = "info"
align = "center"

[[ui.sections.contents.children.children]]
type = "divider"
id = "counter_divider"

[[ui.sections.contents.children.children]]
type = "row"
id = "counter_actions"
gap = "sm"
justify = "center"

[[ui.sections.contents.children.children.children]]
type = "button"
id = "increment_button"
name = "Increment"
variant = "primary"
size = "lg"
action = "counter.increment"

[[ui.sections.contents.children.children.children]]
type = "button"
id = "reset_button"
name = "Reset"
variant = "secondary"
action = "counter.reset"
```

---

## 5.2 Example Binding Configuration (Conceptual)

The following example illustrates how a separate configuration can connect actions and state to UI elements.

```toml id="j4f2kp"
# griffon.bindings.toml

[state]
counter = 0

[[bindings]]
on = "counter.increment"
update = "state.counter = state.counter + 1"

[[bindings.targets]]
id = "counter_value"
property = "name"
value = "state.counter"

[[bindings]]
on = "counter.reset"
update = "state.counter = 0"

[[bindings.targets]]
id = "counter_value"
property = "name"
value = "state.counter"
```

This example is **illustrative only**.
The binding file format may vary depending on the Griffon runtime design.

---

# 6. Plugin Metadata

The `[plugin]` table defines the plugin identity and high-level navigation.

---

## 6.1 Fields

| Field         |             Type | Required | Description                         |
| ------------- | ---------------: | -------: | ----------------------------------- |
| `name`        |           string |      Yes | Human-readable plugin name          |
| `id`          |           string |      Yes | Unique plugin identifier            |
| `version`     |           string |      Yes | Plugin version string               |
| `author`      |           string |      Yes | Plugin author name                  |
| `description` |           string |       No | Short plugin description            |
| `tabs`        | array of strings |      Yes | Ordered list of available tab names |

---

## 6.2 Example

```toml id="r8y6mw"
[plugin]
name = "Antivirus"
id = "org.griffon.antivirus"
version = "1.2.0"
author = "Raphael"
description = "Antivirus protection and threat management"
tabs = ["overview", "threats", "settings"]
```

---

# 7. Sections

A section is a container that groups UI elements under a specific tab.

Multiple sections may belong to the same tab.

---

## 7.1 Section Definition

```toml id="c9m2na"
[[ui.sections]]
id = "overview_main"
tab = "overview"
```

---

## 7.2 Fields

| Field |   Type | Required | Description                             |
| ----- | -----: | -------: | --------------------------------------- |
| `id`  | string |      Yes | Unique section identifier               |
| `tab` | string |      Yes | Name of the tab this section belongs to |

---

## 7.3 Rules

* Section `id` values **must be unique** within the manifest.
* The `tab` value **must** exist in `plugin.tabs`.
* Each section may contain zero or more `[[ui.sections.contents]]` entries.
* Section contents are rendered in declaration order unless overridden by layout rules.

---

# 8. Section Contents

Each `[[ui.sections.contents]]` entry defines a single UI element.

Elements are declared sequentially and belong to the **most recently declared section**.

Container elements may define nested children through `children`.

---

## 8.1 Common Fields

| Field  |   Type |    Required | Description                                   |
| ------ | -----: | ----------: | --------------------------------------------- |
| `type` | string |         Yes | Component type                                |
| `name` | string |     Usually | Display label, title, or primary visible text |
| `id`   | string | Recommended | Unique element identifier                     |

---

## 8.2 Common Rules

* `type` is mandatory for all elements.
* `name` is the primary display string for most components.
* `id` is strongly recommended for any interactive or bindable element.
* Element `id` values should be unique within the manifest.
* Additional fields are interpreted according to the `type`.
* Container components may contain nested `children`.

---

# 9. Component Types

This section defines the base set of generic UI elements supported by the Griffon UI system.

---

# 9.1 `text`

Displays static text, headings, descriptions, or status messages.

---

## Fields

| Field     |     Type |    Required | Description                                       |
| --------- | -------: | ----------: | ------------------------------------------------- |
| `type`    | `"text"` |         Yes | Component type                                    |
| `name`    |   string |         Yes | Text content                                      |
| `id`      |   string | Recommended | Element identifier                                |
| `variant` |   string |          No | `body`, `title`, `subtitle`, `caption`, `status`  |
| `tone`    |   string |          No | `neutral`, `info`, `success`, `warning`, `danger` |
| `align`   |   string |          No | `left`, `center`, `right`                         |

---

## Example

```toml id="n7t3el"
[[ui.sections.contents]]
type = "text"
id = "status_label"
name = "Your device is protected"
variant = "title"
tone = "success"
```

---

# 9.2 `button`

Displays a clickable action button.

A button emits an action identifier through the `action` field.

---

## Fields

| Field        |       Type |    Required | Description                                          |
| ------------ | ---------: | ----------: | ---------------------------------------------------- |
| `type`       | `"button"` |         Yes | Component type                                       |
| `name`       |     string |         Yes | Button label                                         |
| `id`         |     string | Recommended | Element identifier                                   |
| `variant`    |     string |          No | `primary`, `secondary`, `ghost`, `danger`, `success` |
| `size`       |     string |          No | `sm`, `md`, `lg`                                     |
| `disabled`   |       bool |          No | Disabled state                                       |
| `full_width` |       bool |          No | Expands to container width                           |
| `icon`       |     string |          No | Optional icon identifier                             |
| `action`     |     string | Recommended | Semantic action emitted on click                     |

---

## Example

```toml id="v6p4zu"
[[ui.sections.contents]]
type = "button"
id = "run_scan_button"
name = "Run Quick Scan"
variant = "primary"
size = "lg"
action = "scan.quick"
```

---

# 9.3 `select`

Displays a single-choice dropdown list.

A select emits an action identifier when its value changes.

---

## Fields

| Field         |             Type |    Required | Description                       |
| ------------- | ---------------: | ----------: | --------------------------------- |
| `type`        |       `"select"` |         Yes | Component type                    |
| `name`        |           string |         Yes | Label                             |
| `id`          |           string | Recommended | Element identifier                |
| `options`     | array of strings |         Yes | Selectable options                |
| `value`       |           string |          No | Initial/current selected value    |
| `placeholder` |           string |          No | Placeholder text                  |
| `disabled`    |             bool |          No | Disabled state                    |
| `logo`        |           string |          No | Optional visual marker            |
| `action`      |           string | Recommended | Semantic action emitted on change |

---

## Example

```toml id="k5w8hf"
[[ui.sections.contents]]
type = "select"
id = "scan_type_select"
name = "Scan Type"
options = ["Quick", "Full", "Custom"]
value = "Quick"
action = "scan.type.change"
```

---

# 9.4 `input`

Displays a text input field.

---

## Fields

| Field         |      Type |    Required | Description                                 |
| ------------- | --------: | ----------: | ------------------------------------------- |
| `type`        | `"input"` |         Yes | Component type                              |
| `name`        |    string |         Yes | Label                                       |
| `id`          |    string | Recommended | Element identifier                          |
| `value`       |    string |          No | Initial/current value                       |
| `placeholder` |    string |          No | Placeholder text                            |
| `input_type`  |    string |          No | `text`, `search`, `password`, `number`      |
| `disabled`    |      bool |          No | Disabled state                              |
| `action`      |    string | Recommended | Semantic action emitted on change or submit |

---

## Example

```toml id="u2c1ag"
[[ui.sections.contents]]
type = "input"
id = "threat_search"
name = "Search Threats"
placeholder = "Search by name or path..."
input_type = "search"
action = "threats.search"
```

---

# 9.5 `switch`

Displays a binary on/off toggle.

---

## Fields

| Field         |       Type |    Required | Description                       |
| ------------- | ---------: | ----------: | --------------------------------- |
| `type`        | `"switch"` |         Yes | Component type                    |
| `name`        |     string |         Yes | Label                             |
| `id`          |     string | Recommended | Element identifier                |
| `value`       |       bool |         Yes | Current state                     |
| `description` |     string |          No | Secondary explanatory text        |
| `disabled`    |       bool |          No | Disabled state                    |
| `action`      |     string | Recommended | Semantic action emitted on toggle |

---

## Example

```toml id="m9d7qr"
[[ui.sections.contents]]
type = "switch"
id = "realtime_protection"
name = "Real-time Protection"
value = true
description = "Automatically scans files as they are accessed"
action = "protection.realtime.toggle"
```

---

# 9.6 `checkbox`

Displays a checkable boolean option.

---

## Fields

| Field         |         Type |    Required | Description                       |
| ------------- | -----------: | ----------: | --------------------------------- |
| `type`        | `"checkbox"` |         Yes | Component type                    |
| `name`        |       string |         Yes | Label                             |
| `id`          |       string | Recommended | Element identifier                |
| `value`       |         bool |         Yes | Checked state                     |
| `description` |       string |          No | Helper text                       |
| `disabled`    |         bool |          No | Disabled state                    |
| `action`      |       string | Recommended | Semantic action emitted on toggle |

---

## Example

```toml id="f4z1lp"
[[ui.sections.contents]]
type = "checkbox"
id = "include_archives"
name = "Include archived files"
value = false
action = "scan.include_archives.toggle"
```

---

# 9.7 `tabs`

Displays internal tab-like navigation within a section.

---

## Fields

| Field    |             Type |    Required | Description                       |
| -------- | ---------------: | ----------: | --------------------------------- |
| `type`   |         `"tabs"` |         Yes | Component type                    |
| `name`   |           string |         Yes | Label or internal name            |
| `id`     |           string | Recommended | Element identifier                |
| `items`  | array of strings |         Yes | Tab labels                        |
| `value`  |           string |          No | Active tab                        |
| `action` |           string | Recommended | Semantic action emitted on change |

---

## Example

```toml id="z8n5xe"
[[ui.sections.contents]]
type = "tabs"
id = "threat_tabs"
name = "Threat Views"
items = ["Active", "Quarantined", "Resolved"]
value = "Active"
action = "threats.view.change"
```

---

# 9.8 `badge`

Displays a small status indicator or label.

---

## Fields

| Field     |      Type | Required | Description                                       |
| --------- | --------: | -------: | ------------------------------------------------- |
| `type`    | `"badge"` |      Yes | Component type                                    |
| `name`    |    string |      Yes | Badge text                                        |
| `id`      |    string | Optional | Element identifier                                |
| `tone`    |    string |       No | `neutral`, `info`, `success`, `warning`, `danger` |
| `variant` |    string |       No | `solid`, `soft`, `outline`                        |

---

## Example

```toml id="g2s6vc"
[[ui.sections.contents]]
type = "badge"
name = "Protected"
tone = "success"
variant = "soft"
```

---

# 9.9 `progress`

Displays progress for long-running operations.

---

## Fields

| Field        |         Type |    Required | Description                               |
| ------------ | -----------: | ----------: | ----------------------------------------- |
| `type`       | `"progress"` |         Yes | Component type                            |
| `name`       |       string |         Yes | Label                                     |
| `id`         |       string | Recommended | Element identifier                        |
| `value`      |      integer |         Yes | Progress value from 0 to 100              |
| `show_label` |         bool |          No | Whether to show numeric progress          |
| `status`     |       string |          No | `default`, `success`, `warning`, `danger` |
| `animated`   |         bool |          No | Animated progress indicator               |

---

## Example

```toml id="b1j9td"
[[ui.sections.contents]]
type = "progress"
id = "scan_progress"
name = "Current Scan"
value = 42
show_label = true
animated = true
```

---

# 9.10 `card`

Displays a grouped visual container.

A card is primarily a presentational grouping component.

---

## Fields

| Field         |     Type |    Required | Description                                       |
| ------------- | -------: | ----------: | ------------------------------------------------- |
| `type`        | `"card"` |         Yes | Component type                                    |
| `name`        |   string |         Yes | Card title                                        |
| `id`          |   string | Recommended | Element identifier                                |
| `description` |   string |          No | Subtitle or secondary text                        |
| `tone`        |   string |          No | `neutral`, `info`, `success`, `warning`, `danger` |

---

## Example

```toml id="h3l7qy"
[[ui.sections.contents]]
type = "card"
id = "status_card"
name = "Protection Status"
description = "All protection systems are active"
tone = "success"
```

---

# 9.11 `group`

Defines a generic logical container for grouping related elements.

A `group` is the primary generic container for nesting UI components. It can be used for semantic grouping without enforcing a specific layout direction.

---

## Fields

| Field         |      Type |    Required | Description                          |
| ------------- | --------: | ----------: | ------------------------------------ |
| `type`        | `"group"` |         Yes | Component type                       |
| `id`          |    string | Recommended | Element identifier                   |
| `name`        |    string |          No | Optional semantic label              |
| `title`       |    string |          No | Optional visible title               |
| `description` |    string |          No | Optional helper text                 |
| `gap`         |    string |          No | `none`, `xs`, `sm`, `md`, `lg`, `xl` |
| `children`    |     array |         Yes | Nested child components              |

---

## Example

```toml id="t5n4wd"
[[ui.sections.contents]]
type = "group"
id = "scan_controls"
name = "Scan Controls"

[[ui.sections.contents.children]]
type = "button"
id = "quick_scan_button"
name = "Quick Scan"
action = "scan.quick"

[[ui.sections.contents.children]]
type = "button"
id = "full_scan_button"
name = "Full Scan"
action = "scan.full"
```

---

# 9.12 `row`

Defines a horizontal layout container.

A `row` arranges child elements horizontally.

---

## Fields

| Field      |    Type |    Required | Description                                               |
| ---------- | ------: | ----------: | --------------------------------------------------------- |
| `type`     | `"row"` |         Yes | Component type                                            |
| `id`       |  string | Recommended | Element identifier                                        |
| `name`     |  string |          No | Optional semantic label                                   |
| `gap`      |  string |          No | `none`, `xs`, `sm`, `md`, `lg`, `xl`                      |
| `align`    |  string |          No | `start`, `center`, `end`, `stretch`                       |
| `justify`  |  string |          No | `start`, `center`, `end`, `space-between`, `space-around` |
| `wrap`     |    bool |          No | Allow child wrapping                                      |
| `children` |   array |         Yes | Nested child components                                   |

---

## Example

```toml id="p8c2fn"
[[ui.sections.contents]]
type = "row"
id = "header_actions"
gap = "sm"
justify = "space-between"
align = "center"

[[ui.sections.contents.children]]
type = "text"
id = "header_title"
name = "Threats"
variant = "title"

[[ui.sections.contents.children]]
type = "button"
id = "refresh_button"
name = "Refresh"
variant = "secondary"
action = "threats.refresh"
```

---

# 9.13 `column`

Defines a vertical layout container.

A `column` arranges child elements vertically.

---

## Fields

| Field      |       Type |    Required | Description                                               |
| ---------- | ---------: | ----------: | --------------------------------------------------------- |
| `type`     | `"column"` |         Yes | Component type                                            |
| `id`       |     string | Recommended | Element identifier                                        |
| `name`     |     string |          No | Optional semantic label                                   |
| `gap`      |     string |          No | `none`, `xs`, `sm`, `md`, `lg`, `xl`                      |
| `align`    |     string |          No | `start`, `center`, `end`, `stretch`                       |
| `justify`  |     string |          No | `start`, `center`, `end`, `space-between`, `space-around` |
| `children` |      array |         Yes | Nested child components                                   |

---

## Example

```toml id="d7v1ke"
[[ui.sections.contents]]
type = "column"
id = "status_stack"
gap = "md"

[[ui.sections.contents.children]]
type = "text"
id = "status_title"
name = "Protection Status"
variant = "title"

[[ui.sections.contents.children]]
type = "badge"
id = "status_badge"
name = "Protected"
tone = "success"
```

---

# 9.14 `divider`

Displays a visual separator between elements.

A `divider` is a purely presentational component used to break up sections of content.

---

## Fields

| Field         |        Type | Required | Description                                       |
| ------------- | ----------: | -------: | ------------------------------------------------- |
| `type`        | `"divider"` |      Yes | Component type                                    |
| `id`          |      string | Optional | Element identifier                                |
| `orientation` |      string |       No | `horizontal`, `vertical`                          |
| `label`       |      string |       No | Optional divider label                            |
| `tone`        |      string |       No | `neutral`, `info`, `success`, `warning`, `danger` |

---

## Example

```toml id="x1q6zm"
[[ui.sections.contents]]
type = "divider"
id = "section_break"
orientation = "horizontal"
```

---

# 9.15 `table`

Displays structured tabular data.

A `table` is intended for read-only or semi-interactive data presentation. Row-level actions should be handled through separate bindings or explicit action columns.

---

## Fields

| Field        |             Type |    Required | Description                                             |
| ------------ | ---------------: | ----------: | ------------------------------------------------------- |
| `type`       |        `"table"` |         Yes | Component type                                          |
| `id`         |           string | Recommended | Element identifier                                      |
| `name`       |           string |         Yes | Table label or title                                    |
| `columns`    | array of strings |         Yes | Ordered list of column labels                           |
| `rows`       |  array of arrays |          No | Initial row data                                        |
| `striped`    |             bool |          No | Alternate row styling                                   |
| `compact`    |             bool |          No | Reduced spacing                                         |
| `selectable` |             bool |          No | Allow row selection                                     |
| `action`     |           string | Recommended | Semantic action emitted on row selection or interaction |

---

## Example

```toml id="w9m3aj"
[[ui.sections.contents]]
type = "table"
id = "threats_table"
name = "Detected Threats"
columns = ["Name", "Severity", "Location", "Status"]
rows = [
  ["Trojan.Agent", "High", "C:/Downloads/file.exe", "Quarantined"],
  ["PUA.Toolbar", "Low", "C:/Program Files/App", "Ignored"]
]
striped = true
compact = false
selectable = true
action = "threats.table.select"
```

---

# 10. Nested Components

Container components support nested child declarations.

Supported container types:

* `group`
* `row`
* `column`

Nested child components are declared using `children`.

---

## 10.1 Nested Syntax

```toml id="m4t8re"
[[ui.sections.contents]]
type = "row"
id = "actions_row"

[[ui.sections.contents.children]]
type = "button"
id = "save_button"
name = "Save"

[[ui.sections.contents.children]]
type = "button"
id = "cancel_button"
name = "Cancel"
```

---

## 10.2 Deep Nesting

Nested containers may themselves contain `children`.

```toml id="y2n6ph"
[[ui.sections.contents]]
type = "group"
id = "panel"

[[ui.sections.contents.children]]
type = "column"
id = "panel_stack"

[[ui.sections.contents.children.children]]
type = "text"
id = "panel_title"
name = "Settings"

[[ui.sections.contents.children.children]]
type = "row"
id = "panel_actions"

[[ui.sections.contents.children.children.children]]
type = "button"
id = "save_button"
name = "Save"
```

---

# 11. Action Semantics

Interactive elements should emit **semantic action identifiers** rather than implementation-specific handler names.

Examples:

* `counter.increment`
* `counter.reset`
* `scan.quick`
* `scan.type.change`
* `protection.realtime.toggle`
* `quarantine.delete`

This convention improves portability, maintainability, and binding clarity.

---

# 12. Identifier Recommendations

Element identifiers (`id`) are strongly recommended for:

* interactive elements
* dynamic text values
* values updated by bindings
* elements referenced by automation or tests

## Recommended naming style

Use lowercase snake_case:

* `counter_value`
* `increment_button`
* `scan_progress`
* `realtime_protection`

---

# 13. Recommended Semantic Tones

Tones communicate semantic meaning across the UI.

| Tone      | Meaning                       | Typical Usage               |
| --------- | ----------------------------- | --------------------------- |
| `neutral` | Default state                 | Standard UI elements        |
| `info`    | Informational state           | Active process, syncing     |
| `success` | Healthy or completed state    | Protected, connected        |
| `warning` | Attention required            | Outdated, partial issue     |
| `danger`  | Critical or destructive state | Threat found, delete action |

---

# 14. Recommended Variants

## 14.1 Text variants

* `title`
* `subtitle`
* `body`
* `caption`
* `status`

## 14.2 Button variants

* `primary`
* `secondary`
* `ghost`
* `danger`
* `success`

## 14.3 Badge variants

* `solid`
* `soft`
* `outline`

---

# 15. Full Example: Antivirus Overview

```toml id="e6k2ws"
[plugin]
name = "Antivirus"
id = "org.griffon.antivirus"
version = "1.0.0"
author = "Raphael"
description = "Device protection and threat management"
tabs = ["overview", "threats", "settings"]

[[ui.sections]]
id = "overview_main"
tab = "overview"

[[ui.sections.contents]]
type = "column"
id = "overview_layout"
gap = "md"

[[ui.sections.contents.children]]
type = "row"
id = "overview_header"
justify = "space-between"
align = "center"

[[ui.sections.contents.children.children]]
type = "text"
id = "overview_title"
name = "Your device is protected"
variant = "title"
tone = "success"

[[ui.sections.contents.children.children]]
type = "badge"
id = "protection_badge"
name = "Protected"
tone = "success"
variant = "soft"

[[ui.sections.contents.children]]
type = "divider"
id = "overview_divider"

[[ui.sections.contents.children]]
type = "group"
id = "scan_controls"
name = "Scan Controls"

[[ui.sections.contents.children.children]]
type = "row"
id = "scan_actions"
gap = "sm"

[[ui.sections.contents.children.children.children]]
type = "button"
id = "quick_scan_button"
name = "Run Quick Scan"
variant = "primary"
size = "lg"
action = "scan.quick"

[[ui.sections.contents.children.children.children]]
type = "button"
id = "full_scan_button"
name = "Run Full Scan"
variant = "secondary"
action = "scan.full"

[[ui.sections.contents.children]]
type = "select"
id = "scan_type_select"
name = "Scan Type"
options = ["Quick", "Full", "Custom"]
value = "Quick"
action = "scan.type.change"

[[ui.sections.contents.children]]
type = "progress"
id = "scan_progress"
name = "Current Scan"
value = 42
show_label = true
animated = true

[[ui.sections.contents.children]]
type = "switch"
id = "realtime_protection"
name = "Real-time Protection"
value = true
description = "Automatically scans files as they are opened or downloaded"
action = "protection.realtime.toggle"

[[ui.sections]]
id = "threats_main"
tab = "threats"

[[ui.sections.contents]]
type = "table"
id = "threats_table"
name = "Detected Threats"
columns = ["Name", "Severity", "Location", "Status"]
rows = [
  ["Trojan.Agent", "High", "C:/Downloads/file.exe", "Quarantined"],
  ["PUA.Toolbar", "Low", "C:/Program Files/App", "Ignored"]
]
striped = true
selectable = true
action = "threats.table.select"
```

---

# 16. Validation Rules

The following validation rules are recommended for Griffon UI manifests.

---

## 16.1 Global rules

* `[plugin]` must exist exactly once
* `plugin.tabs` must contain at least one value
* `[[ui.sections]]` may appear zero or more times
* each `ui.sections.id` must be unique
* each `ui.sections.tab` must exist in `plugin.tabs`

---

## 16.2 Element rules

* each `[[ui.sections.contents]]` must appear after a `[[ui.sections]]`
* each element must define `type`
* most elements should define `name`
* interactive elements should define `action`
* bindable elements should define `id`
* `progress.value` should be clamped or validated to `0..100`
* `select.value` should match one of `options` when provided
* `tabs.value` should match one of `items` when provided
* `table.rows` should match the length of `table.columns`
* `divider` does not require `name`
* `group`, `row`, and `column` may omit `name`
* `group`, `row`, and `column` should contain at least one child
* nested `children` declarations must belong to a container element

---

# 17. Best Practices

## 17.1 Prefer generic components

Avoid domain-specific component types such as:

* `threat_card`
* `scan_button`
* `vpn_status_box`

Prefer:

* `card`
* `button`
* `badge`
* `progress`
* `switch`
* `group`
* `row`
* `column`
* `table`

---

## 17.2 Keep action names semantic

Preferred:

```toml id="u7b9cn"
action = "scan.quick"
```

Avoid:

```toml id="n1f4za"
action = "runQuickScanEngineV2"
```

---

## 17.3 Assign `id` values to dynamic elements

Any element whose visible value may change should have an `id`.

Examples:

* counter values
* progress indicators
* status labels
* result fields
* table selections

---

## 17.4 Use layout containers for structure

Recommended layout strategy:

* use `column` for vertical page flow
* use `row` for horizontal action groups
* use `group` for semantic grouping
* use `divider` for visual separation
* use `table` for structured datasets

---

## 17.5 Keep UI manifests presentation-only

Do not embed logic semantics directly into UI structure beyond `action` identifiers and initial values.

---

# 18. Future Extensions

The following component types are recommended as future additions:

* `grid`
* `list`
* `modal`
* `drawer`
* `toast`
* `timeline`
* `metric`
* `chart`

These additions would enable richer layouts while preserving the same declarative architecture.

---

# 19. Minimal Recommended Component Set

A minimal yet practical Griffon UI implementation should support:

1. `text`
2. `button`
3. `select`
4. `input`
5. `switch`
6. `checkbox`
7. `badge`
8. `progress`
9. `card`
10. `group`
11. `row`
12. `column`
13. `divider`
14. `table`

This set is sufficient to build:

* counters
* antivirus dashboards
* updater interfaces
* VPN controls
* cleanup tools
* settings panels
* backup modules

---

# 20. Summary

The Griffon UI Manifest Specification defines a generic, reusable, and declarative UI format for plugin-based applications.

The core architecture is based on:

* **TOML-defined UI**
* **generic reusable components**
* **semantic actions**
* **nested layout containers**
* **structured data components**
* **separate binding/logic configuration**
* **modular sections organized by tabs**

This model supports both simple interfaces (such as a counter) and more complex modules (such as antivirus or system utilities) without requiring domain-specific UI types.

---
