use console::style;
use peeko::registry::client::{PlatformParam, RegistryClient, RegistryError};

use crate::config;
use crate::error::{PeekoCliError, Result};
use crate::utils;

const DEFAULT_REGISTRY: &str = "https://registry-1.docker.io";

pub async fn execute(image_url: &str) -> Result<()> {
    let (registry_url, image, tag) = parse_image_url(image_url)?;
    utils::print_header(&format!("Pulling {image}:{tag} from {registry_url}"));

    let mut client = RegistryClient::new(&registry_url).enable_progress();
    client.set_concurrent_downloads(config::get_concurrent_downloads());
    client.set_downloads_dir(config::get_peeko_dir());

    let platform = PlatformParam {
        architecture: None,
        os: None,
        variant: None,
    };

    match client.download_image(&image, &tag, platform).await {
        Ok(_) => {
            utils::print_success(&format!("Successfully pulled {image}:{tag}"));

            let image_path = format!("{image}/{tag}");
            utils::print_info(&format!("Image saved to: {}", style(&image_path).cyan()));
            Ok(())
        }
        Err(RegistryError::ManifestNotFound) => {
            utils::print_error(&format!("Image not found for {image}:{tag}"));
            Err(PeekoCliError::RuntimeError("".to_string()))
        }
        Err(err) => {
            utils::print_error(&format!("Failed to pull {image}:{tag}"));
            Err(err.into())
        }
    }
}

fn parse_image_url(image_url: &str) -> Result<(String, String, String)> {
    let (image_url, tag) = image_url
        .rsplit_once(':')
        .ok_or_else(|| PeekoCliError::Input("Image tag is required".to_string()))?;

    match image_url.find('/') {
        Some(index) => {
            let registry = &image_url[..index];
            match registry.find('.') {
                Some(_) => {
                    let image = &image_url[index + 1..];
                    Ok((
                        format!("https://{registry}"),
                        image.to_string(),
                        tag.to_string(),
                    ))
                }
                None => Ok((
                    DEFAULT_REGISTRY.to_string(),
                    image_url.to_string(),
                    tag.to_string(),
                )),
            }
        }
        None => {
            let image = format!("library/{image_url}");
            Ok((DEFAULT_REGISTRY.to_string(), image, tag.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_image_url() {
        // full image url
        let (registry_url, image, tag) =
            parse_image_url("registry-1.docker.io/library/nginx:latest").unwrap();
        assert_eq!(registry_url, "https://registry-1.docker.io");
        assert_eq!(image, "library/nginx");
        assert_eq!(tag, "latest");

        // official image
        let (registry_url, image, tag) = parse_image_url("nginx:latest").unwrap();
        assert_eq!(registry_url, DEFAULT_REGISTRY);
        assert_eq!(image, "library/nginx");
        assert_eq!(tag, "latest");

        let (registry_url, image, tag) = parse_image_url("mcp/slack:latest").unwrap();
        assert_eq!(registry_url, DEFAULT_REGISTRY);
        assert_eq!(image, "mcp/slack");
        assert_eq!(tag, "latest");

        let (registry_url, image, tag) = parse_image_url("ghcr.io/owner/image:tag").unwrap();
        assert_eq!(registry_url, "https://ghcr.io");
        assert_eq!(image, "owner/image");
        assert_eq!(tag, "tag");
    }
}
