use std::env;
use std::path::PathBuf;

use dirs;

const DEFAULT_PEEKO_DIR: &str = "~/.peeko";
const DEFAULT_CONCURRENT_DOWNLOADS: &str = "4";

pub fn get_peeko_dir() -> PathBuf {
    let peeko_dir = env::var("PEEKO_DIR").unwrap_or(DEFAULT_PEEKO_DIR.to_string());
    if peeko_dir.starts_with("~") {
        let home_dir = dirs::home_dir();
        if let Some(home_dir) = home_dir {
            return home_dir.join(&peeko_dir[2..]);
        }
    }

    peeko_dir.into()
}

pub fn get_concurrent_downloads() -> usize {
    let concurrent_downloads =
        env::var("CONCURRENT_DOWNLOADS").unwrap_or(DEFAULT_CONCURRENT_DOWNLOADS.to_string());
    concurrent_downloads.parse().unwrap_or(3)
}
