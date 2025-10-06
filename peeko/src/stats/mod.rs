use crate::reader::vfs::{FileEntry, VirtualFileSystem};

pub fn show_statistics(vfs: &VirtualFileSystem) {
    let entries = vfs.get_entries();

    let mut file_count = 0;
    let mut dir_count = 0;
    let mut symlink_count = 0;
    let mut total_size = 0u64;

    for entry in entries.values() {
        match entry {
            FileEntry::File { size, .. } => {
                file_count += 1;
                total_size += size;
            }
            FileEntry::Directory { .. } => dir_count += 1,
            FileEntry::Symlink { .. } => symlink_count += 1,
        }
    }

    println!("\n=== Filesystem Statistics ===");
    println!("Total directories: {}", dir_count);
    println!("Total files: {}", file_count);
    println!("Total symlinks: {}", symlink_count);
    println!(
        "Total size: {:.2} MB",
        total_size as f64 / (1024.0 * 1024.0)
    );
}

pub fn show_tree(vfs: &VirtualFileSystem, max_depth: usize, max_items: usize) {
    println!(
        "\n=== Directory Tree (depth={}, max_items={}) ===",
        max_depth, max_items
    );
    let tree = vfs.get_directory_tree(max_depth);
    tree.print(max_items);
}

pub fn list_top_level(vfs: &VirtualFileSystem) {
    println!("\n=== Top-level Entries ===");

    let entries = vfs.get_entries();
    let mut top_level: Vec<_> = entries
        .keys()
        .filter(|path| path.components().count() == 1)
        .collect();

    top_level.sort();

    for path in top_level {
        if let Some(entry) = entries.get(path) {
            match entry {
                FileEntry::Directory { .. } => println!("  /{}/", path.display()),
                FileEntry::File { size, .. } => {
                    println!("  /{} ({} bytes)", path.display(), size)
                }
                FileEntry::Symlink { target, .. } => {
                    println!("  /{} -> {}", path.display(), target)
                }
            }
        }
    }
}
