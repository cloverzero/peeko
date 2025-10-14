//! Lightweight virtual filesystem for materialising file listings in OCI images.

use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

use super::dir_tree::DirectoryTree;

/// Metadata recorded for each entry tracked by the virtual filesystem.
#[derive(Debug, Clone)]
pub enum FileEntry {
    /// Regular file along with its size and layer index.
    File { size: u64, layer_index: usize },
    /// Directory created in the given layer.
   Directory { layer_index: usize },
    /// Symbolic link pointing at `target`.
   Symlink { target: String, layer_index: usize },
}

/// In-memory index of filesystem entries extracted from image layers.
pub struct VirtualFileSystem {
    // 路径 -> 文件条目
    entries: HashMap<PathBuf, FileEntry>,
}

impl VirtualFileSystem {
    /// Creates an empty virtual filesystem.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Inserts or replaces the entry stored at `path`.
    pub fn add_entry(&mut self, path: PathBuf, entry: FileEntry) {
        self.entries.insert(path, entry);
    }

    /// Returns the metadata for a given path if it exists.
    pub fn get_entry<P: AsRef<Path>>(&self, path: P) -> Option<&FileEntry> {
        self.entries.get(path.as_ref())
    }

    /// Deletes the entry at `path`.
    pub fn delete_entry(&mut self, path: &PathBuf) {
        self.entries.remove(path);
    }

    /// Removes all entries contained inside the directory `dir`.
    pub fn clear_directory(&mut self, dir: &Path) {
        let dir_str = dir.to_string_lossy();
        let dir_prefix = format!("{dir_str}/");
        self.entries
            .retain(|path, _| !path.to_string_lossy().starts_with(&dir_prefix));
    }

    /// Returns a view of the raw entry map.
    pub fn get_entries(&self) -> &HashMap<PathBuf, FileEntry> {
        &self.entries
    }

    /// Builds a `DirectoryTree` covering all tracked paths.
    pub fn get_directory_tree(&self) -> DirectoryTree {
        let tree = DirectoryTree::new();

        for (path, file_entry) in &self.entries {
            let is_dir = matches!(file_entry, FileEntry::Directory { .. });
            tree.add_path(path, is_dir);
        }

        tree
    }
}

impl Default for VirtualFileSystem {
    fn default() -> Self {
        Self::new()
    }
}
