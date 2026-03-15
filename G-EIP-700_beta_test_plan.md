---
title:          Beta Test Plan
subtitle:       Griffon, security engine (beta)
author:         Boitel Alexis, Raphael Mabille
module:         G-EIP-700
version:        1.0
---

<!--# **BETA TEST PLAN – Griffon**-->

## **1. Project context**

Griffon is a modular cybersecurity solution for Linux desktop systems.
It is composed of:

- A **Tauri + Next.js desktop application** (UI layer)
- A **Rust daemon** (security engine) responsible for loading and executing security plugins

The desktop application acts solely as an interface.
It communicates with the daemon to:

- Load and list available security plugins
- Call plugin-exposed methods through a unified interface
- Display results, logs, history, and notifications

The UI could be replaced by a CLI without modifying the core engine.

For the beta version (v1.0), the platform provides two main modules:

1. **Static Malware Analyzer**
   Analyzes files, folders, binaries, and archive files using YARA rules.

2. **System Cleaner**
   Analyzes and removes cache, temporary files, and unnecessary system data.

---

## **2. User Roles**

The following roles will be involved in beta testing.

| **Role Name** | **Description** |
|--------------|----------------|
| Standard User | Runs analyses and cleaning tasks using default settings |
| Advanced User | Configures plugins, performance options, history, and notifications |
| Developer | Creates, installs, and tests custom plugins |

---

## **3. Feature table**

All of the listed features will be demonstrated during the beta presentation.

| **Feature ID** | **User role** | **Feature name** | **Short description** |
|--------------|---------------|------------------|----------------------|
| F1 | Everyone | Launch application | Application starts without crashing |
| F2 | Everyone | Plugin listing | All installed plugins are displayed |
| F3 | Everyone | Cleaner data selection | Select types of data to analyze and clean |
| F4 | Everyone | Cleaner analysis | Analyze system to find removable data |
| F5 | Everyone | Cleaner results | Display detected data before cleaning |
| F6 | Everyone | Cleaner execution | Select and remove detected data |
| F7 | Everyone | File or folder analysis | Choose a file or folder to analyze |
| F8 | Everyone | Default static analysis | Run analysis with default settings |
| F9 | Advanced | Threat selection | Choose types of threats to detect |
| F10 | Advanced | Performance configuration | Configure number of threads |
| F11 | Everyone | Analysis results | Display duration, files analyzed, threats found |
| F12 | Everyone | Threat handling | Delete or confine detected threats |
| F13 | Everyone | Archive analysis | Analyze compressed archive files |
| F14 | Everyone | Plugins history | View history of each plugins past results |
| F15 | Everyone | Notifications | Receive notification when analysis ends |
| F16 | Advanced | Plugin management | Enable or disable plugins |
| F17 | Advanced | Plugin installation | Install or uninstall plugins |
| F18 | Advanced | Notification settings | Enable or disable notifications per plugin |
| F19 | Developer | Plugin creation | Create a plugin using documentation |

---

## **4. Success criteria**

Success criteria are based on **real usage scenarios** performed by beta testers.

| **Feature ID** | **Key success criteria** | **Indicator / Use case** | **Result** |
|--------------|--------------------------|--------------------------|------------|
| F1 | Application is stable | Application launches repeatedly without crash | Achieved |
| F2 | Plugins are correctly detected | All installed plugins appear at startup | Achieved |
| F3 | Cleaner configuration works | Selected data types match analysis scope | Achieved |
| F4 | Cleaner analysis completes | Analysis finishes and returns results | Achieved |
| F5 | Cleaner results are accurate | Displayed data corresponds to actual files | Achieved |
| F6 | Cleaning is controlled | Only selected data is removed | Achieved |
| F7 | File/folder selection works | Correct target is analyzed | Achieved |
| F8 | Default analysis is usable | Analysis completes without configuration | Achieved |
| F9 | Threat filters apply | Only selected threat types are reported | Achieved |
| F10 | Performance settings apply | Thread count affects analysis behavior | Achieved |
| F11 | Results are understandable | User can interpret analysis outcome | Achieved |
| F12 | Threat actions are effective | Files are deleted or confined correctly | Achieved |
| F13 | Archive analysis is supported | Archive contents are analyzed correctly | Achieved |
| F14 | History is preserved | Past analyses remain accessible for every plugins | Achieved |
| F15 | Background notification works | Notification received after task completion | Achieved |
| F16 | Plugin control is effective | Disabled plugins cannot be executed | Achieved |
| F17 | Plugin lifecycle is functional | Plugins can be installed and removed | Achieved |
| F18 | Notification preferences apply | Disabled notifications are not sent | Achieved |
| F19 | Developer workflow is usable | Plugin created using documentation only | Achieved |
