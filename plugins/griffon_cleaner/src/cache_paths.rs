// src/cache_paths.rs

use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
pub enum CacheCategory {
    System,
    User,
    Browser,
    DevTools,
    PackageManager,
    DesktopEnv,
}

/// Un chemin de cache "connu" par Griffon.
/// pattern : chemin brut, avec éventuellement "~" en début.
#[derive(Debug, Clone, Copy)]
pub struct CachePath {
    pub id: &'static str,
    pub category: CacheCategory,
    pub pattern: &'static str,
}

pub const KNOWN_CACHE_PATHS: &[CachePath] = &[
    // =======================
    // SYSTÈME
    // =======================
    CachePath {
        id: "system_var_cache",
        category: CacheCategory::System,
        pattern: "/var/cache",
    },
    CachePath {
        id: "system_tmp",
        category: CacheCategory::System,
        pattern: "/tmp",
    },
    CachePath {
        id: "system_var_tmp",
        category: CacheCategory::System,
        pattern: "/var/tmp",
    },
    CachePath {
        id: "system_machine_journal",
        category: CacheCategory::System,
        pattern: "/var/log/journal",
    },

    // =======================
    // PACKAGE MANAGERS
    // =======================
    CachePath {
        id: "apt_lists",
        category: CacheCategory::PackageManager,
        pattern: "/var/lib/apt/lists",
    },
    CachePath {
        id: "apt_archives",
        category: CacheCategory::PackageManager,
        pattern: "/var/cache/apt/archives",
    },
    CachePath {
        id: "dnf_cache",
        category: CacheCategory::PackageManager,
        pattern: "/var/cache/dnf",
    },
    CachePath {
        id: "pacman_pkg",
        category: CacheCategory::PackageManager,
        pattern: "/var/cache/pacman/pkg",
    },
    CachePath {
        id: "snap_cache",
        category: CacheCategory::PackageManager,
        pattern: "/var/lib/snapd/snaps",
    },

    // =======================
    // UTILISATEUR (HOME)
    // =======================
    CachePath {
        id: "user_cache",
        category: CacheCategory::User,
        pattern: "~/.cache",
    },
    CachePath {
        id: "user_trash",
        category: CacheCategory::User,
        pattern: "~/.local/share/Trash",
    },
    CachePath {
        id: "user_thumbnails",
        category: CacheCategory::User,
        pattern: "~/.thumbnails",
    },
    CachePath {
        id: "user_downloads_tmp",
        category: CacheCategory::User,
        pattern: "~/Downloads",
    },

    // =======================
    // DEV TOOLS / LANGAGES
    // =======================
    CachePath {
        id: "pip_cache",
        category: CacheCategory::DevTools,
        pattern: "~/.cache/pip",
    },
    CachePath {
        id: "npm_cache",
        category: CacheCategory::DevTools,
        pattern: "~/.npm",
    },
    CachePath {
        id: "yarn_cache",
        category: CacheCategory::DevTools,
        pattern: "~/.cache/yarn",
    },
    CachePath {
        id: "cargo_registry_cache",
        category: CacheCategory::DevTools,
        pattern: "~/.cargo/registry/cache",
    },
    CachePath {
        id: "cargo_git_db",
        category: CacheCategory::DevTools,
        pattern: "~/.cargo/git/db",
    },
    CachePath {
        id: "rustup_toolchains",
        category: CacheCategory::DevTools,
        pattern: "~/.rustup/toolchains",
    },
    CachePath {
        id: "rustup_downloads",
        category: CacheCategory::DevTools,
        pattern: "~/.rustup/downloads",
    },
    CachePath {
        id: "pipenv_cache",
        category: CacheCategory::DevTools,
        pattern: "~/.cache/pipenv",
    },

    // =======================
    // NAVIGATEURS
    // =======================
    CachePath {
        id: "firefox_profile_cache",
        category: CacheCategory::Browser,
        pattern: "~/.cache/mozilla/firefox",
    },
    CachePath {
        id: "firefox_profiles",
        category: CacheCategory::Browser,
        pattern: "~/.mozilla/firefox",
    },
    CachePath {
        id: "chrome_cache",
        category: CacheCategory::Browser,
        pattern: "~/.cache/google-chrome",
    },
    CachePath {
        id: "chromium_cache",
        category: CacheCategory::Browser,
        pattern: "~/.cache/chromium",
    },
    CachePath {
        id: "brave_cache",
        category: CacheCategory::Browser,
        pattern: "~/.cache/BraveSoftware",
    },
    CachePath {
        id: "vivaldi_cache",
        category: CacheCategory::Browser,
        pattern: "~/.cache/vivaldi",
    },

    // =======================
    // ENVIRONNEMENTS GRAPHIQUES / APPS
    // =======================
    CachePath {
        id: "gnome_cache",
        category: CacheCategory::DesktopEnv,
        pattern: "~/.cache/gnome-software",
    },
    CachePath {
        id: "flatpak_cache",
        category: CacheCategory::DesktopEnv,
        pattern: "~/.var/app",
    },
    CachePath {
        id: "kde_cache",
        category: CacheCategory::DesktopEnv,
        pattern: "~/.cache/kioexec",
    },
    CachePath {
        id: "vlc_cache",
        category: CacheCategory::DesktopEnv,
        pattern: "~/.cache/vlc",
    },
    CachePath {
        id: "spotify_cache",
        category: CacheCategory::DesktopEnv,
        pattern: "~/.cache/spotify",
    },
];

/// Remplace le "~" par $HOME si présent.
/// Si $HOME n'existe pas, on retourne None.
pub fn expand_home(pattern: &str) -> Option<PathBuf> {
    use std::env;

    if let Some(stripped) = pattern.strip_prefix("~/") {
        let home = env::var_os("HOME")?;
        Some(PathBuf::from(home).join(stripped))
    } else {
        Some(PathBuf::from(pattern))
    }
}
