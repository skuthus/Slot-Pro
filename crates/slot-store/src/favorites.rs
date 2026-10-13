use std::path::{Path, PathBuf};

use crate::atomic::atomic_write;
use crate::platform::Platform;

fn favorites_path(root: &Path) -> PathBuf {
    root.join("Config").join("favorites.txt")
}

pub fn read_favorites(root: &Path) -> Vec<(Platform, String)> {
    let Ok(text) = std::fs::read_to_string(favorites_path(root)) else {
        return Vec::new();
    };
    text.lines()
        .filter_map(|line| {
            let (dir, stem) = line.trim_end_matches('\r').split_once('/')?;
            let platform = Platform::ALL
                .into_iter()
                .find(|p| p.dir_name().eq_ignore_ascii_case(dir))?;
            (!stem.is_empty()).then(|| (platform, stem.to_string()))
        })
        .collect()
}

pub fn write_favorites(root: &Path, favorites: &[(Platform, String)]) -> std::io::Result<()> {
    let text: String = favorites
        .iter()
        .map(|(platform, stem)| format!("{}/{stem}\n", platform.dir_name()))
        .collect();
    atomic_write(&favorites_path(root), text.as_bytes())
}
