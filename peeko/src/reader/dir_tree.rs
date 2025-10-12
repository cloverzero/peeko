use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::{Rc, Weak};

#[derive(Debug)]
pub struct TreeNode {
    pub name: String,
    pub children: RefCell<HashMap<String, Rc<TreeNode>>>,
    pub parent: RefCell<Weak<TreeNode>>,
    pub is_dir: bool,
}

impl TreeNode {
    pub fn pwd(&self, with_root: bool) -> String {
        let mut components = vec![self.name.clone()];
        let mut current = self.parent.borrow().upgrade();

        while let Some(parent) = current {
            components.push(parent.name.clone());
            current = parent.parent.borrow().upgrade();
        }

        // remove the root node
        components.pop();
        components.reverse();

        if with_root {
            format!("/{}", components.join("/"))
        } else {
            components.join("/")
        }
    }

    pub fn print(&self, depth: usize, max_depth: usize, is_last: bool, prefix: &str) {
        let new_prefix = if depth == 0 {
            println!("{}", &self.name);
            prefix.to_string()
        } else {
            let connector = if is_last { "└── " } else { "├── " };
            println!(
                "{}{}{}{}",
                prefix,
                connector,
                &self.name,
                if self.is_dir { "/" } else { "" }
            );
            format!("{}{}", prefix, if is_last { "    " } else { "│   " })
        };

        // print the current node, then check if the depth is greater than the max depth
        if depth > max_depth {
            return;
        }

        let children = self.children.borrow();
        let mut sorted_children: Vec<&Rc<TreeNode>> = children.values().collect();
        let total = sorted_children.len();
        sorted_children.sort_by(|a, b| a.name.cmp(&b.name));
        for (index, child) in sorted_children.iter().enumerate() {
            let is_last = index == total - 1;
            child.print(depth + 1, max_depth, is_last, &new_prefix);
        }
    }
}

#[derive(Debug)]
pub struct DirectoryTree {
    pub root: Rc<TreeNode>,
}

impl DirectoryTree {
    pub fn new() -> Self {
        Self {
            root: Rc::new(TreeNode {
                name: "/".to_string(),
                children: RefCell::new(HashMap::new()),
                parent: RefCell::new(Weak::new()),
                is_dir: true,
            }),
        }
    }

    pub fn add_path<P: AsRef<Path>>(&self, path: P, is_dir: bool) {
        let mut components: Vec<_> = path
            .as_ref()
            .components()
            .filter_map(|c| c.as_os_str().to_str())
            .collect();

        if components.is_empty() {
            return;
        }
        if components[0].eq("/") {
            components.remove(0);
        }

        let mut current = Rc::clone(&self.root);

        let last_index = components.len() - 1;

        for (index, component) in components.iter().enumerate() {
            let new_node = Rc::clone(
                current
                    .children
                    .borrow_mut()
                    .entry(component.to_string())
                    .or_insert_with(|| {
                        Rc::new(TreeNode {
                            name: component.to_string(),
                            children: RefCell::new(HashMap::new()),
                            parent: RefCell::new(Rc::downgrade(&current)),
                            is_dir: index != last_index || is_dir,
                        })
                    }),
            );

            current = new_node;
        }
    }

    pub fn find(&self, path: &str) -> Option<Rc<TreeNode>> {
        if path.eq("/") {
            return Some(Rc::clone(&self.root));
        }
        let components: Vec<_> = path.trim_matches('/').split('/').collect();
        if components.is_empty() {
            return None;
        }

        let mut current = Rc::clone(&self.root);
        for component in components {
            let node = current
                .children
                .borrow()
                .get(component)
                .map(Rc::clone);
            match node {
                Some(node) => {
                    current = node;
                }
                None => {
                    return None;
                }
            }
        }

        Some(current)
    }

    pub fn print(&self, max_depth: usize) {
        self.root.print(0, max_depth, true, "");
    }
}

impl Default for DirectoryTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_path() {
        let tree = DirectoryTree::new();
        let path = "/usr/bin/ls";
        tree.add_path(path, false);
        let node = tree.find(path).unwrap();
        assert_eq!(node.pwd(true), path);
    }
}
