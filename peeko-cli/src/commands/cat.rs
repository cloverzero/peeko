use std::time::Duration;
use tokio::io::{self, AsyncWriteExt};

use indicatif::{ProgressBar, ProgressStyle};
use peeko::reader::build_image_reader;

use crate::error::{PeekoCliError, Result};
use crate::utils;

pub async fn execute(image_with_tag: &str, path: &str) -> Result<()> {
    match image_with_tag.rsplit_once(':') {
        Some((image, tag)) => {
            let image_path = peeko::config::get_peeko_dir().join(format!("{image}/{tag}"));
            // Check if image exists
            if !std::path::Path::new(&image_path).exists() {
                utils::print_error(&format!("Image {image}:{tag} not found locally"));
                utils::print_info("Use 'peeko pull' to download the image first.");
                return Err(PeekoCliError::RuntimeError("".to_string()));
            }

            // 创建一个无限 spinner
            let pb = utils::SpinnerGuard::new(ProgressBar::new_spinner());
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} {msg}")
                    .unwrap()
                    .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ "), // 这些字符会循环形成动画
            );
            pb.set_message("Loading image...");
            pb.enable_steady_tick(Duration::from_millis(100));

            let reader = build_image_reader(&image_path).await?;

            let file_path = if let Some(stripped) = path.strip_prefix('/') {
                stripped
            } else {
                path
            };

            let bytes = reader.read_file(file_path).await?;
            pb.finish_and_clear();

            io::stdout().write_all(&bytes).await?;
            Ok(())
        }
        None => Err(PeekoCliError::RuntimeError(
            "Image with tag is required".to_string(),
        )),
    }
}
