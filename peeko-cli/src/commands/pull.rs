use anyhow::Result;
use console::style;
use peeko::registry::client::{PlatformParam, RegistryClient};

use crate::utils;

pub async fn execute(image: &str, tag: &str, registry_url: &str) -> Result<()> {
    utils::print_header(&format!("Pulling {}:{} from {}", image, tag, registry_url));

    let mut client = RegistryClient::new(registry_url).enable_progress();

    let platform = PlatformParam {
        architecture: None,
        os: None,
        variant: None,
    };

    match client.download_image(image, tag, platform).await {
        Ok(_) => {
            utils::print_success(&format!("Successfully pulled {}:{}", image, tag));

            let image_path = format!("{}/{}", image, tag);
            utils::print_info(&format!("Image saved to: {}", style(&image_path).cyan()));
        }
        Err(e) => {
            utils::print_error(&format!("Failed to pull image: {}", e));
            return Err(e);
        }
    }

    Ok(())
}
