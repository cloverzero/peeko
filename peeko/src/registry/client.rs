use anyhow;
use futures_util::{StreamExt, TryStreamExt, stream};
use reqwest;
use serde::{Deserialize, Serialize};
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;

use super::manifest::{Manifest, ManifestList};

#[derive(Debug, Deserialize, Serialize)]
struct TokenResponse {
    pub token: Option<String>,
    pub access_token: Option<String>,
    pub expires_in: Option<u64>,
}

pub struct PlatformParam {
    architecture: Option<String>,
    os: Option<String>,
    variant: Option<String>,
}

const MAX_CONCURRENT_DOWNLOADS: usize = 3;

#[derive(Clone)]
pub struct RegistryClient {
    http: reqwest::Client,
    registry_url: String,
    auth_token: Option<String>,
    username: Option<String>,
    password: Option<String>,
}

impl RegistryClient {
    pub fn new(registry_url: &str) -> Self {
        Self {
            http: reqwest::Client::new(),
            registry_url: registry_url.to_string(),
            auth_token: None,
            username: None,
            password: None,
        }
    }

    pub fn with_credentials(registry_url: &str, username: &str, password: &str) -> Self {
        let mut client = Self::new(registry_url);
        client.username = Some(username.to_string());
        client.password = Some(password.to_string());
        client
    }

    pub fn with_token(registry_url: &str, token: &str) -> Self {
        let mut client = Self::new(registry_url);
        client.auth_token = Some(token.to_string());
        client
    }

    async fn authenticate_if_needed(&mut self, url: &str) -> anyhow::Result<()> {
        if self.auth_token.is_some() {
            return Ok(());
        }

        let response = self.http.head(url).send().await?;

        if response.status() == 401 {
            let auth_header = response
                .headers()
                .get("www-authenticate")
                .ok_or_else(|| anyhow::anyhow!("No www-authenticate header found"))?
                .to_str()?;

            // 解析类似：Bearer realm="https://auth.docker.io/token",service="registry.docker.io",scope="repository:library/nginx:pull"
            let mut realm = String::new();
            let mut service = String::new();
            let mut scope = None;

            // 简单的解析逻辑（生产环境建议使用更健壮的解析器）
            for part in auth_header.split(',') {
                let part = part.trim();
                if part.starts_with("Bearer realm=") {
                    realm = part
                        .strip_prefix("Bearer realm=\"")
                        .and_then(|s| s.strip_suffix('"'))
                        .unwrap_or("")
                        .to_string();
                } else if part.starts_with("service=") {
                    service = part
                        .strip_prefix("service=\"")
                        .and_then(|s| s.strip_suffix('"'))
                        .unwrap_or("")
                        .to_string();
                } else if part.starts_with("scope=") {
                    scope = part
                        .strip_prefix("scope=\"")
                        .and_then(|s| s.strip_suffix('"'))
                        .map(|s| s.to_string());
                }
            }

            let mut token_url = format!("{}?service={}", realm, service);
            if let Some(scope) = scope {
                token_url = format!("{}&scope={}", token_url, scope);
            }

            let mut request = self.http.get(token_url);
            if let (Some(username), Some(password)) = (&self.username, &self.password) {
                request = request.basic_auth(username, Some(password));
            }

            let response = request.send().await?;

            if !response.status().is_success() {
                return Err(anyhow::anyhow!(
                    "Failed to get token: HTTP {}",
                    response.status()
                ));
            }

            let auth_response: TokenResponse = response.json().await?;
            let token = auth_response
                .token
                .or(auth_response.access_token)
                .ok_or_else(|| anyhow::anyhow!("No token found"))?;
            self.auth_token = Some(token);
        }

        Ok(())
    }

    fn with_auth(&self, mut request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(token) = &self.auth_token {
            request = request.bearer_auth(token);
        } else if let (Some(username), Some(password)) = (&self.username, &self.password) {
            request = request.basic_auth(username, Some(password));
        }
        request
    }

    pub async fn get_image_manifest(
        &mut self,
        image: &str,
        tag_or_digest: &str,
    ) -> anyhow::Result<Manifest> {
        let url = format!(
            "{}/v2/{}/manifests/{}",
            self.registry_url, image, tag_or_digest
        );

        self.authenticate_if_needed(&url).await?;

        let response = self
            .with_auth(self.http.get(&url).header(
                "Accept",
                "application/vnd.docker.distribution.manifest.v2+json",
            ))
            .send()
            .await?;

        let content_type = response
            .headers()
            .get("content-type")
            .ok_or_else(|| anyhow::anyhow!("No content-type header found"))?
            .to_str()?;

        match content_type {
            "application/vnd.oci.image.manifest.v1+json"
            | "application/vnd.docker.distribution.manifest.v2+json" => {
                Ok(Manifest::OCIManifest(response.json().await?))
            }
            "application/vnd.oci.image.index.v1+json"
            | "application/vnd.docker.distribution.manifest.list.v2+json" => {
                Ok(Manifest::OCIIndex(response.json().await?))
            }
            _ => Err(anyhow::anyhow!(
                "Unsupported content type: {}",
                content_type
            )),
        }
    }

    pub async fn download_image(
        &mut self,
        image: &str,
        tag: &str,
        platform: PlatformParam,
    ) -> anyhow::Result<()> {
        let manifest = self.get_image_manifest(image, tag).await?;

        let image_manifest = match manifest {
            Manifest::OCIManifest(oci_manifest) => Some(oci_manifest),
            Manifest::OCIIndex(manifest_list) => {
                let target = self.match_manifest(&manifest_list, &platform);
                match target {
                    Some(target) => {
                        let manifest = self.get_image_manifest(image, &target.digest).await?;
                        if let Manifest::OCIManifest(oci_manifest) = manifest {
                            Some(oci_manifest)
                        } else {
                            None
                        }
                    }
                    None => None,
                }
            }
        };

        let oci_manifest =
            image_manifest.ok_or_else(|| anyhow::anyhow!("No image manifest found"))?;

        let folder_path = format!("{}/{}", image, tag);
        fs::create_dir_all(&folder_path).await?;

        let tasks = oci_manifest
            .layers
            .iter()
            .map(|layer| self.download_layer(image, &layer.digest, &folder_path));

        stream::iter(tasks)
            .buffer_unordered(MAX_CONCURRENT_DOWNLOADS)
            .try_collect::<Vec<_>>()
            .await?;

        Ok(())
    }

    async fn download_layer(
        &self,
        image: &str,
        digest: &str,
        dest_path: &str,
    ) -> anyhow::Result<()> {
        let url = format!("{}/v2/{}/blobs/{}", self.registry_url, image, digest);
        println!("Downloading layer: {}", url);
        let response = self.with_auth(self.http.get(url)).send().await?;
        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to download layer: HTTP {}",
                response.status()
            ));
        }

        let mut file = File::create(format!("{}/{}", dest_path, digest)).await?;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
        }

        file.flush().await?;

        Ok(())
    }

    fn match_manifest<'a>(
        &self,
        manifest_list: &'a ManifestList,
        platform: &PlatformParam,
    ) -> Option<&'a super::manifest::PlatformManifest> {
        if let (None, None, None) = (&platform.architecture, &platform.os, &platform.variant) {
            let first = manifest_list.manifests.first();
            return first;
        }
        let target = manifest_list.manifests.iter().find(|m| {
            if let Some(arch) = &platform.architecture {
                if m.platform.architecture.ne(arch) {
                    return false;
                }
            }
            if let Some(os) = &platform.os {
                if m.platform.os.ne(os) {
                    return false;
                }
            }
            if let (Some(variant), Some(m_variant)) = (&platform.variant, &m.platform.variant) {
                if variant.ne(m_variant) {
                    return false;
                }
            }

            true
        });

        target
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_image_manifest() {
        let mut client = RegistryClient::new("https://registry-1.docker.io");
        let image_manifest = client
            .get_image_manifest("library/hello-world", "latest")
            .await
            .unwrap();
        println!("Image manifest: {:?}", image_manifest);
    }

    #[tokio::test]
    async fn test_download_image() {
        let mut client = RegistryClient::new("https://registry-1.docker.io");
        client
            .download_image(
                "library/node",
                "24-alpine",
                PlatformParam {
                    architecture: None,
                    os: None,
                    variant: None,
                },
            )
            .await
            .unwrap();
    }
}
