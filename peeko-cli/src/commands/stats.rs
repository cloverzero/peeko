use anyhow::Result;
use peeko::reader::image_reader::ImageReader;
use peeko::stats;

use crate::utils;

pub async fn execute(image: &str, tag: &str) -> Result<()> {
    utils::print_header(&format!("Statistics for {}:{}", image, tag));

    let image_path = format!("{}/{}", image, tag);

    // Check if image exists
    if !std::path::Path::new(&image_path).exists() {
        utils::print_error(&format!("Image {}:{} not found locally", image, tag));
        utils::print_info("Use 'peeko pull' to download the image first.");
        return Ok(());
    }

    let mut reader = ImageReader::new(&image_path);

    match reader.reconstruct().await {
        Ok(vfs) => {
            // Show detailed statistics
            stats::show_statistics(&vfs);

            // Show top-level directory listing
            stats::list_top_level(&vfs);

            // Show a brief tree view
            println!();
            utils::print_info("Directory structure preview (depth=2, max 5 items per level):");
            stats::show_tree(&vfs, 2, 5);
        }
        Err(e) => {
            utils::print_error(&format!("Failed to reconstruct filesystem: {}", e));
            return Err(e);
        }
    }

    Ok(())
}