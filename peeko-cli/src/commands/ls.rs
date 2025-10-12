use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};
use peeko::reader::{build_image_reader, vfs::FileEntry};
use tabled::{Table, Tabled, settings::Style};

use crate::error::{PeekoCliError, Result};
use crate::utils;

#[derive(Tabled)]
struct FileInfo {
    #[tabled(rename = "Type")]
    file_type: String,
    #[tabled(rename = "Size")]
    size: String,
    #[tabled(rename = "File")]
    name: String,
}

pub async fn execute(image_with_tag: &str, path: &str) -> Result<()> {
    match image_with_tag.rsplit_once(':') {
        Some((image, tag)) => {
            let image_path = peeko::config::get_peeko_dir().join(format!("{image}/{tag}"));
            // Check if image exists
            if !std::path::Path::new(&image_path).exists() {
                utils::print_warning(&format!("Image {image}:{tag} not found locally"));
                utils::print_info("Use 'peeko pull' to download the image first.");
                return Err(PeekoCliError::RuntimeError("".to_string()));
            }

            // 创建一个无限 spinner
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} {msg}")
                    .unwrap()
                    .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ "), // 这些字符会循环形成动画
            );
            pb.set_message("Loading image...");
            pb.enable_steady_tick(Duration::from_millis(100));

            let reader = build_image_reader(&image_path).await?;

            let dir_tree = reader.get_dir_tree()?;
            let target_node = dir_tree.find(path);
            match target_node {
                Some(node) => {
                    let mut files: Vec<FileInfo> = Vec::new();
                    for child in node.children.borrow().values() {
                        let full_path_str = child.pwd(false);
                        let entry = reader.get_file_meatadata(&full_path_str);
                        if let Some(entry) = entry {
                            let file_info = match entry {
                                FileEntry::File { size, .. } => FileInfo {
                                    name: child.name.clone(),
                                    size: utils::format_size(*size),
                                    file_type: "file".to_string(),
                                },
                                FileEntry::Directory { .. } => FileInfo {
                                    name: child.name.clone(),
                                    size: "".to_string(),
                                    file_type: "dir".to_string(),
                                },
                                FileEntry::Symlink { .. } => FileInfo {
                                    name: child.name.clone(),
                                    size: "".to_string(),
                                    file_type: "symlink".to_string(),
                                },
                            };
                            files.push(file_info);
                        }
                    }

                    files.sort_by(|a, b| a.name.cmp(&b.name));

                    let len = files.len();
                    let mut table = Table::new(files);
                    table.with(Style::blank());

                    pb.finish_and_clear();
                    println!("{table}");
                    println!();
                    utils::print_info(&format!("Showing {len} files"));
                    Ok(())
                }
                None => {
                    pb.finish_and_clear();
                    Err(PeekoCliError::RuntimeError(format!(
                        "Path {path} not found"
                    )))
                }
            }
        }
        None => Err(PeekoCliError::InputError(
            "Image tag is required".to_string(),
        )),
    }
}
