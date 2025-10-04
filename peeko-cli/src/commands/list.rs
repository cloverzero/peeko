use anyhow::Result;
use console::style;
use std::fs;
use tabled::{Table, Tabled};

use crate::utils;

#[derive(Tabled)]
struct ImageInfo {
    #[tabled(rename = "Image")]
    name: String,
    #[tabled(rename = "Tag")]
    tag: String,
    #[tabled(rename = "Size")]
    size: String,
    #[tabled(rename = "Status")]
    status: String,
}

pub async fn execute() -> Result<()> {
    utils::print_header("Downloaded Images");

    let mut images = Vec::new();

    // Scan for downloaded images
    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                let image_name = entry.file_name().to_string_lossy().to_string();

                // Skip non-image directories
                if image_name.starts_with('.') || image_name == "target" {
                    continue;
                }

                if let Ok(tag_entries) = fs::read_dir(entry.path()) {
                    for tag_entry in tag_entries.flatten() {
                        if tag_entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                            let tag_name = tag_entry.file_name().to_string_lossy().to_string();

                            // Check if manifest.json exists
                            let manifest_path = tag_entry.path().join("manifest.json");
                            if manifest_path.exists() {
                                let size = calculate_directory_size(&tag_entry.path()).unwrap_or(0);

                                images.push(ImageInfo {
                                    name: image_name.clone(),
                                    tag: tag_name,
                                    size: utils::format_size(size),
                                    status: style("✅ Ready").green().to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    if images.is_empty() {
        utils::print_info("No downloaded images found.");
        utils::print_info("Use 'peeko pull <image>' to download an image.");
    } else {
        let len = images.len();
        let table = Table::new(images);
        println!("{}", table);
        println!();
        utils::print_info(&format!("Found {} downloaded image(s)", len));
    }

    Ok(())
}

fn calculate_directory_size(path: &std::path::Path) -> Result<u64> {
    let mut total_size = 0;

    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                total_size += entry.metadata()?.len();
            } else if path.is_dir() {
                total_size += calculate_directory_size(&path)?;
            }
        }
    }

    Ok(total_size)
}
