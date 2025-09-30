use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
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

    pub fn add_entry(&mut self, path: PathBuf, entry: FileEntry) {
        // 如果文件已被标记为删除，不添加
        if !self.deleted.contains(&path) {
            self.entries.insert(path, entry);
        }
    }

    pub fn delete_entry(&mut self, path: &PathBuf) {
        self.entries.remove(path);
        self.deleted.insert(path.to_path_buf());
    }

    pub fn clear_directory(&mut self, dir: &Path) {
        let dir_str = dir.to_string_lossy();

        // 移除该目录下的所有条目
        let dir_prefix = format!("{}/", dir_str);
        self.entries
            .retain(|path, _| !path.to_string_lossy().starts_with(&dir_prefix));

        // 标记所有子路径为已删除
        for path in self.entries.keys() {
            if path.starts_with(dir) {
                self.deleted.insert(path.clone());
            }
        }
    }

    pub fn get_entries(&self) -> &HashMap<PathBuf, FileEntry> {
        &self.entries
    }

    pub fn get_directory_tree(&self, max_depth: usize) -> DirectoryTree {
        let mut tree = DirectoryTree::new();

        for path in self.entries.keys() {
            tree.add_path(path, max_depth);
        }

        tree
    }
}

#[derive(Debug)]
pub struct DirectoryTree {
    root: TreeNode,
}

#[derive(Debug)]
pub struct TreeNode {
    name: String,
    children: HashMap<String, TreeNode>,
    is_file: bool,
}

impl DirectoryTree {
    pub fn new() -> Self {
        Self {
            root: TreeNode {
                name: "/".to_string(),
                children: HashMap::new(),
                is_file: false,
            },
        }
    }

    pub fn add_path(&mut self, path: &Path, max_depth: usize) {
        let components: Vec<_> = path
            .components()
            .filter_map(|c| c.as_os_str().to_str())
            .collect();

        if components.is_empty() || components.len() > max_depth {
            return;
        }

        let mut current = &mut self.root;
        let last_idx = components.len() - 1;

        for (idx, component) in components.iter().enumerate() {
            let is_last = idx == last_idx;
            current = current
                .children
                .entry(component.to_string())
                .or_insert_with(|| TreeNode {
                    name: component.to_string(),
                    children: HashMap::new(),
                    is_file: is_last,
                });
        }
    }

    pub fn print(&self, max_items_per_level: usize) {
        self.print_node(&self.root, "", true, max_items_per_level, 0);
    }

    fn print_node(
        &self,
        node: &TreeNode,
        prefix: &str,
        is_last: bool,
        max_items: usize,
        shown_count: usize,
    ) {
        if shown_count > 0 {
            let connector = if is_last { "└── " } else { "├── " };
            let symbol = if node.is_file { "" } else { "/" };
            println!("{}{}{}{}", prefix, connector, node.name, symbol);
        }

        if !node.is_file {
            let new_prefix = if shown_count == 0 {
                prefix.to_string()
            } else {
                format!("{}{}", prefix, if is_last { "    " } else { "│   " })
            };

            let mut children: Vec<_> = node.children.values().collect();
            children.sort_by(|a, b| a.name.cmp(&b.name));

            let total_children = children.len();
            let display_count = max_items.min(total_children);

            for (idx, child) in children.iter().take(display_count).enumerate() {
                let is_last_child = idx == display_count - 1 && total_children <= max_items;
                self.print_node(
                    child,
                    &new_prefix,
                    is_last_child,
                    max_items,
                    shown_count + 1,
                );
            }

            if total_children > max_items {
                let remaining = total_children - max_items;
                println!("{}... and {} more items", new_prefix, remaining);
            }
        }
    }
}
