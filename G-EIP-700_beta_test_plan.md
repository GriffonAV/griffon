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
| F1 | Everyone | Launch application | Start the application without encountering crashes or system errors. |
| F2 | Everyone | Plugin listing | Display a dynamic list of all currently installed plugins. |
| F3 | Everyone | Cleaner data selection | Select specific types of system data (cache, temporary files) for analysis. |
| F4 | Everyone | Cleaner analysis | Scan the system to identify removable or unnecessary data. |
| F5 | Everyone | Cleaner results | Review the detected data details before proceeding with cleaning. |
| F6 | Everyone | Cleaner execution | Remove the selected system data to free up space. |
| F7 | Everyone | File or folder analysis | Target a specific file or directory for a security scan. |
| F8 | Everyone | Default static analysis | Run a malware analysis using the predefined default settings. |
| F9 | Advanced | Threat selection | Filter the specific types of threats to be detected during analysis. |
| F10 | Advanced | Performance configuration | Adjust the number of threads to optimize analysis speed and behavior. |
| F11 | Everyone | Analysis results | View the analysis summary, including duration and threats found. |
| F12 | Everyone | Threat handling | Manage detected threats by deleting or confining them to quarantine. |
| F13 | Everyone | Archive analysis | Toggle the inclusion of compressed archive files (e.g., .zip) within the analysis scope. |
| F14 | Everyone | Plugins history | Access the historical logs and past results for every installed plugin. |
| F15 | Everyone | Notifications | Receive a system alert once an analysis or task is completed. |
| F16 | Advanced | Plugin management | Toggle the enabled or disabled status of specific plugins. |
| F17 | Advanced | Plugin installation | Handle the complete lifecycle of plugins, including installation and removal. |
| F18 | Advanced | Notification settings | Configure notification preferences on a per-plugin basis. |
| F19 | Developer | Plugin creation | Build new custom plugins using the provided technical documentation. |

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
