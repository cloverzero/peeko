use anyhow::Result;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use peeko::registry::client::{RegistryClient, PlatformParam};

use crate::utils;

pub async fn execute(image: &str, tag: &str, registry_url: &str) -> Result<()> {
    utils::print_header(&format!("Pulling {}:{} from {}", image, tag, registry_url));

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.blue} {msg}")
            .unwrap(),
    );

    pb.set_message("Connecting to registry...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let mut client = RegistryClient::new(registry_url);

    pb.set_message("Downloading image...");

    let platform = PlatformParam {
        architecture: None,
        os: None,
        variant: None,
    };

    match client.download_image(image, tag, platform).await {
        Ok(_) => {
            pb.finish_and_clear();
            utils::print_success(&format!("Successfully pulled {}:{}", image, tag));

            let image_path = format!("{}/{}", image, tag);
            utils::print_info(&format!("Image saved to: {}", style(&image_path).cyan()));
        }
        Err(e) => {
            pb.finish_and_clear();
            utils::print_error(&format!("Failed to pull image: {}", e));
            return Err(e);
        }
    }

    Ok(())
}