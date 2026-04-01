use crate::Profile;
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

#[derive(Debug, Clone, Copy)]
pub struct CachePath {
    pub id: &'static str,
    pub category: CacheCategory,
    pub pattern: &'static str,

    pub requires_root: bool,
    pub dangerous: bool,
    pub safe_in_profile: bool,
}

pub const KNOWN_CACHE_PATHS: &[CachePath] = &[
    // =======================
    // SYSTÈME
    // =======================
    CachePath {
        id: "system_var_cache",
        category: CacheCategory::System,
        pattern: "/var/cache",
        requires_root: true,
        dangerous: false,
        safe_in_profile: true,
    },
    CachePath {
        id: "system_tmp",
        category: CacheCategory::System,
        pattern: "/tmp",
        requires_root: false,
        dangerous: false,
        safe_in_profile: true,
    },
    CachePath {
        id: "system_var_tmp",
        category: CacheCategory::System,
        pattern: "/var/tmp",
        requires_root: true,
        dangerous: false,
        safe_in_profile: true,
    },
    CachePath {
        id: "system_machine_journal",
        category: CacheCategory::System,
        pattern: "/var/log/journal",
        requires_root: true,
        dangerous: true,
        safe_in_profile: false,
    },
    // =======================
    // PACKAGE MANAGERS
    // =======================
    CachePath {
        id: "apt_lists",
        category: CacheCategory::PackageManager,
        pattern: "/var/lib/apt/lists",
        requires_root: true,
        dangerous: false,
        safe_in_profile: true,
    },
    CachePath {
        id: "apt_archives",
        category: CacheCategory::PackageManager,
        pattern: "/var/cache/apt/archives",
        requires_root: true,
        dangerous: false,
        safe_in_profile: true,
    },
    CachePath {
        id: "dnf_cache",
        category: CacheCategory::PackageManager,
        pattern: "/var/cache/dnf",
        requires_root: true,
        dangerous: false,
        safe_in_profile: true,
    },
    CachePath {
        id: "pacman_pkg",
        category: CacheCategory::PackageManager,
        pattern: "/var/cache/pacman/pkg",
        requires_root: true,
        dangerous: false,
        safe_in_profile: true,
    },
    CachePath {
        id: "snap_cache",
        category: CacheCategory::PackageManager,
        pattern: "/var/lib/snapd/snaps",
        requires_root: true,
        dangerous: true,
        safe_in_profile: false,
    },
    // =======================
    // UTILISATEUR (HOME)
    // =======================
    CachePath {
        id: "user_cache",
        category: CacheCategory::User,
        pattern: "~/.cache",
        requires_root: false,
        dangerous: false,
        safe_in_profile: true,
    },
    CachePath {
        id: "user_trash",
        category: CacheCategory::User,
        pattern: "~/.local/share/Trash",
        requires_root: false,
        dangerous: false,
        safe_in_profile: true,
    },
    CachePath {
        id: "user_thumbnails",
        category: CacheCategory::User,
        pattern: "~/.thumbnails",
        requires_root: false,
        dangerous: false,
        safe_in_profile: true,
    },
    CachePath {
        id: "user_downloads_tmp",
        category: CacheCategory::User,
        pattern: "~/Downloads",
        requires_root: false,
        dangerous: true,
        safe_in_profile: false,
    },
    // =======================
    // DEV TOOLS / LANGAGES
    // =======================
    CachePath {
        id: "pip_cache",
        category: CacheCategory::DevTools,
        pattern: "~/.cache/pip",
        requires_root: false,
        dangerous: false,
        safe_in_profile: true,
    },
    CachePath {
        id: "npm_cache",
        category: CacheCategory::DevTools,
        pattern: "~/.npm",
        requires_root: false,
        dangerous: false,
        safe_in_profile: true,
    },
    CachePath {
        id: "yarn_cache",
        category: CacheCategory::DevTools,
        pattern: "~/.cache/yarn",
        requires_root: false,
        dangerous: false,
        safe_in_profile: true,
    },
    CachePath {
        id: "cargo_registry_cache",
        category: CacheCategory::DevTools,
        pattern: "~/.cargo/registry/cache",
        requires_root: false,
        dangerous: false,
        safe_in_profile: true,
    },
    CachePath {
        id: "cargo_git_db",
        category: CacheCategory::DevTools,
        pattern: "~/.cargo/git/db",
        requires_root: false,
        dangerous: false,
        safe_in_profile: true,
    },
    CachePath {
        id: "rustup_toolchains",
        category: CacheCategory::DevTools,
        pattern: "~/.rustup/toolchains",
        requires_root: false,
        dangerous: true,
        safe_in_profile: false,
    },
    CachePath {
        id: "rustup_downloads",
        category: CacheCategory::DevTools,
        pattern: "~/.rustup/downloads",
        requires_root: false,
        dangerous: false,
        safe_in_profile: true,
    },
    CachePath {
        id: "pipenv_cache",
        category: CacheCategory::DevTools,
        pattern: "~/.cache/pipenv",
        requires_root: false,
        dangerous: false,
        safe_in_profile: true,
    },
    // =======================
    // NAVIGATEURS
    // =======================
    CachePath {
        id: "firefox_profile_cache",
        category: CacheCategory::Browser,
        pattern: "~/.cache/mozilla/firefox",
        requires_root: false,
        dangerous: false,
        safe_in_profile: true,
    },
    CachePath {
        id: "firefox_profiles",
        category: CacheCategory::Browser,
        pattern: "~/.mozilla/firefox",
        requires_root: false,
        dangerous: true,
        safe_in_profile: false,
    },
    CachePath {
        id: "chrome_cache",
        category: CacheCategory::Browser,
        pattern: "~/.cache/google-chrome",
        requires_root: false,
        dangerous: false,
        safe_in_profile: true,
    },
    CachePath {
        id: "chromium_cache",
        category: CacheCategory::Browser,
        pattern: "~/.cache/chromium",
        requires_root: false,
        dangerous: false,
        safe_in_profile: true,
    },
    CachePath {
        id: "brave_cache",
        category: CacheCategory::Browser,
        pattern: "~/.cache/BraveSoftware",
        requires_root: false,
        dangerous: false,
        safe_in_profile: true,
    },
    CachePath {
        id: "vivaldi_cache",
        category: CacheCategory::Browser,
        pattern: "~/.cache/vivaldi",
        requires_root: false,
        dangerous: false,
        safe_in_profile: true,
    },
    // =======================
    // ENVIRONNEMENTS GRAPHIQUES / APPS
    // =======================
    CachePath {
        id: "gnome_cache",
        category: CacheCategory::DesktopEnv,
        pattern: "~/.cache/gnome-software",
        requires_root: false,
        dangerous: false,
        safe_in_profile: true,
    },
    CachePath {
        id: "flatpak_cache",
        category: CacheCategory::DesktopEnv,
        pattern: "~/.var/app",
        requires_root: false,
        dangerous: true,
        safe_in_profile: false,
    },
    CachePath {
        id: "kde_cache",
        category: CacheCategory::DesktopEnv,
        pattern: "~/.cache/kioexec",
        requires_root: false,
        dangerous: false,
        safe_in_profile: true,
    },
    CachePath {
        id: "vlc_cache",
        category: CacheCategory::DesktopEnv,
        pattern: "~/.cache/vlc",
        requires_root: false,
        dangerous: false,
        safe_in_profile: true,
    },
    CachePath {
        id: "spotify_cache",
        category: CacheCategory::DesktopEnv,
        pattern: "~/.cache/spotify",
        requires_root: false,
        dangerous: false,
        safe_in_profile: true,
    },
];

pub fn expand_home(pattern: &str) -> Option<PathBuf> {
    use std::env;

    if let Some(stripped) = pattern.strip_prefix("~/") {
        let home = env::var_os("HOME")?;
        Some(PathBuf::from(home).join(stripped))
    } else {
        Some(PathBuf::from(pattern))
    }
}

pub fn path_allowed_in_profile(cache: &CachePath, profile: &Profile) -> bool {
    match profile {
        Profile::Safe => cache.safe_in_profile,
        Profile::Full => true,
    }
}
