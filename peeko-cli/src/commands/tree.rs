use anyhow::Result;
use peeko::reader::image_reader::ImageReader;

use crate::utils;

pub async fn execute(image: &str, tag: &str, depth: usize, max_items: usize) -> Result<()> {
    utils::print_header(&format!("Filesystem Tree for {}:{}", image, tag));

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
            let tree = vfs.get_directory_tree(depth);
            tree.print(max_items);

            println!();
            utils::print_info(&format!(
                "Showing directory tree with max depth {} and {} items per level",
                depth, max_items
            ));
        }
        Err(e) => {
            utils::print_error(&format!("Failed to reconstruct filesystem: {}", e));
            return Err(e);
        }
    }

    Ok(())
}