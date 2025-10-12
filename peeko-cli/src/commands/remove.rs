use crate::error::{PeekoCliError, Result};
use crate::utils;

pub async fn execute(image_with_tag: &str) -> Result<()> {
    match image_with_tag.rsplit_once(':') {
        Some((image, tag)) => {
            peeko::fs::delete_image(image, tag)?;
            utils::print_success(&format!("Successfully removed {}", image_with_tag));
            Ok(())
        }
        None => Err(PeekoCliError::InputError(
            "Image tag is required".to_string(),
        )),
    }
}
