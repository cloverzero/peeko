use anyhow::Result;
use peeko::reader::build_image_reader;

use crate::utils;

pub async fn execute(image_with_tag: &str, depth: usize, path: Option<String>) -> Result<()> {
    match image_with_tag.rsplit_once(':') {
        Some((image, tag)) => {
            utils::print_header(&format!("Filesystem Tree for {}:{}", image, tag));

            let image_path = peeko::config::get_peeko_dir().join(format!("{}/{}", image, tag));

            // Check if image exists
            if !std::path::Path::new(&image_path).exists() {
                utils::print_error(&format!("Image {}:{} not found locally", image, tag));
                utils::print_info("Use 'peeko pull' to download the image first.");
                return Ok(());
            }

            let reader = build_image_reader(&image_path).await?;

            match reader.print_dir_tree(depth, path) {
                Ok(()) => {
                    println!();
                    utils::print_info(&format!("Showing directory tree with max depth {}", depth));
                }
                Err(e) => {
                    utils::print_error(&format!("Error printing directory tree: {}", e));
                }
            }
        }
        None => {
            utils::print_warning("Image with tag is required");
        }
    }

    Ok(())
}
