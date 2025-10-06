use anyhow::Result;
use std::fs;
use tabled::{Table, Tabled};

use peeko::config;

use crate::utils;

#[derive(Tabled)]
struct ImageInfo {
    #[tabled(rename = "Image")]
    name: String,
    #[tabled(rename = "Tag")]
    tag: String,
    #[tabled(rename = "Size")]
    size: String,
}

pub async fn execute() -> Result<()> {
    utils::print_header("Downloaded Images");
    let peeko_dir = config::get_peeko_dir();
    let image_directories = peeko::fs::collect_image_directories(&peeko_dir)?;

    let images: Vec<ImageInfo> = image_directories
        .iter()
        .filter_map(|dir| {
            let size = calculate_directory_size(dir).unwrap_or(0);
            let relative_path = dir
                .strip_prefix(&peeko_dir)
                .expect("Must be a subdirectory of the peeko directory")
                .to_string_lossy()
                .to_string();
            if let Some((image, tag)) = relative_path.rsplit_once('/') {
                Some(ImageInfo {
                    name: image.to_owned(),
                    tag: tag.to_owned(),
                    size: utils::format_size(size),
                })
            } else {
                None
            }
        })
        .collect();

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
