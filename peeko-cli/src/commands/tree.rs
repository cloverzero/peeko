use peeko::reader::build_image_reader;

use crate::error::{PeekoCliError, Result};
use crate::utils;

pub async fn execute(image_with_tag: &str, depth: usize, path: Option<String>) -> Result<()> {
    match image_with_tag.rsplit_once(':') {
        Some((image, tag)) => {
            utils::print_header(&format!("Filesystem Tree for {image}:{tag}"));

            let image_path = peeko::config::get_peeko_dir().join(format!("{image}/{tag}"));

            // Check if image exists
            if !std::path::Path::new(&image_path).exists() {
                utils::print_error(&format!("Image {image}:{tag} not found locally"));
                utils::print_info("Use 'peeko pull' to download the image first.");
                return Err(PeekoCliError::RuntimeError("".to_string()));
            }

            let reader = build_image_reader(&image_path).await?;
            reader.print_dir_tree(depth, path)?;

            Ok(())
        }
        None => Err(PeekoCliError::Input(
            "Image with tag is required".to_string(),
        )),
    }
}
