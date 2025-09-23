use anyhow;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

use crate::registry::manifest;

#[derive(Debug, Deserialize, Serialize)]
struct TokenResponse {
    pub token: Option<String>,
    pub access_token: Option<String>,
    pub expires_in: Option<u64>,
}

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
    ) -> anyhow::Result<manifest::Manifest> {
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
            "application/vnd.docker.distribution.manifest.v2+json" => {
                let image_manifest: manifest::ImageManifest = response.json().await?;
                Ok(manifest::Manifest::ImageManifest(image_manifest))
            }
            "application/vnd.docker.distribution.manifest.list.v2+json" => {
                let manifest_list: manifest::ManifestList = response.json().await?;
                Ok(manifest::Manifest::ManifestList(manifest_list))
            }
            "application/vnd.oci.image.manifest.v1+json" => {
                let image_manifest: manifest::ImageManifest = response.json().await?;
                Ok(manifest::Manifest::OCIManifest(image_manifest))
            }
            "application/vnd.oci.image.index.v1+json" => {
                let manifest_list: manifest::ManifestList = response.json().await?;
                Ok(manifest::Manifest::OCIIndex(manifest_list))
            }
            _ => Err(anyhow::anyhow!(
                "Unsupported content type: {}",
                content_type
            )),
        }
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
}
