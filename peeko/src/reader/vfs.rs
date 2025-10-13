use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

use super::dir_tree::DirectoryTree;

#[derive(Debug, Clone)]
pub enum FileEntry {
    File { size: u64, layer_index: usize },
    Directory { layer_index: usize },
    Symlink { target: String, layer_index: usize },
}

pub struct VirtualFileSystem {
    // 路径 -> 文件条目
    entries: HashMap<PathBuf, FileEntry>,
}

impl VirtualFileSystem {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn add_entry(&mut self, path: PathBuf, entry: FileEntry) {
        self.entries.insert(path, entry);
    }

    pub fn get_entry<P: AsRef<Path>>(&self, path: P) -> Option<&FileEntry> {
        self.entries.get(path.as_ref())
    }

    pub fn delete_entry(&mut self, path: &PathBuf) {
        self.entries.remove(path);
    }

    pub fn clear_directory(&mut self, dir: &Path) {
        let dir_str = dir.to_string_lossy();
        let dir_prefix = format!("{dir_str}/");
        self.entries
            .retain(|path, _| !path.to_string_lossy().starts_with(&dir_prefix));
    }

    pub fn get_entries(&self) -> &HashMap<PathBuf, FileEntry> {
        &self.entries
    }

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
