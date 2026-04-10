use std::path::PathBuf;

const APP_DIR_NAME: &str = env!("CARGO_PKG_NAME");

pub(super) fn app_data_dir() -> Option<PathBuf> {
    platform_app_data_root().map(|root| root.join(APP_DIR_NAME))
}

#[cfg(target_os = "macos")]
fn platform_app_data_root() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join("Library").join("Application Support"))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn platform_app_data_root() -> Option<PathBuf> {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .map(|home| home.join(".local").join("share"))
        })
}

#[cfg(windows)]
fn platform_app_data_root() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(PathBuf::from).or_else(|| {
        std::env::var_os("USERPROFILE")
            .map(PathBuf::from)
            .map(|home| home.join("AppData").join("Roaming"))
    })
}
