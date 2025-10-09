use std::path::PathBuf;

use anyhow::Result;
use peeko::reader::ImageReader;

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

            let mut reader = ImageReader::new(&image_path);
            reader.reconstruct().await?;
            println!("path: {:?}", path);
            let dir_tree = reader.get_dir_tree()?;
            let target_node = dir_tree.find(path);
            match target_node {
                Some(node) => {
                    for child in node.children.borrow().values() {
                        let full_path_str = child.pwd(false);
                        let entry = reader.get_file_meatadata(&full_path_str);
                        match entry {
                            Some(entry) => {
                                println!("{} - {:?}", &child.name, entry);
                            }
                            None => {
                                println!("NG: {}", full_path_str);
                            }
                        }
                    }
                }
                None => {
                    utils::print_error(&format!("Path {} not found", path));
                }
            }
        }
        None => {
            utils::print_error("Image with tag is required");
        }
    }
    Ok(())
}
