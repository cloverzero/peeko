use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum FileEntry {
    File { size: u64, layer_index: usize },
    Directory { layer_index: usize },
    Symlink { target: String, layer_index: usize },
}

pub struct VirtualFileSystem {
    // 路径 -> 文件条目
    entries: HashMap<PathBuf, FileEntry>,
    // 记录被删除的文件（whiteout）
    deleted: HashSet<PathBuf>,
}

impl VirtualFileSystem {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            deleted: HashSet::new(),
        }
    }
}
