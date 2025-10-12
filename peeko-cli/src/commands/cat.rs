use std::time::Duration;

use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use peeko::reader::build_image_reader;

use crate::utils;

pub async fn execute(image_with_tag: &str, path: &str) -> Result<()> {
    match image_with_tag.rsplit_once(':') {
        Some((image, tag)) => {
            let image_path = peeko::config::get_peeko_dir().join(format!("{}/{}", image, tag));
            // Check if image exists
            if !std::path::Path::new(&image_path).exists() {
                utils::print_error(&format!("Image {}:{} not found locally", image, tag));
                utils::print_info("Use 'peeko pull' to download the image first.");
                return Ok(());
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

            let file_path = if path.starts_with('/') {
                &path[1..]
            } else {
                path
            };

            let content = reader.read_file(file_path).await?;
            pb.finish_and_clear();

            println!("{}", content);
        }
        None => {
            utils::print_error("Image with tag is required");
        }
    }
    Ok(())
}
