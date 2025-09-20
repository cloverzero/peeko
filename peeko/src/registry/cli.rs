use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

#[derive(Clone)]
pub struct RegistryClient {
    client: Client,
    registry_url: String,
    auth_token: Option<String>,
    username: Option<String>,
    password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub config: LayerInfo,
    pub layers: Vec<LayerInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ManifestList {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub manifests: Vec<ManifestEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ManifestEntry {
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub size: u64,
    pub digest: String,
    pub platform: Option<Platform>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Platform {
    pub architecture: String,
    pub os: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LayerInfo {
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub size: u64,
    pub digest: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthToken {
    pub token: String,
    #[serde(rename = "access_token")]
    pub access_token: Option<String>,
    #[serde(rename = "expires_in")]
    pub expires_in: Option<u64>,
}

#[derive(Debug)]
pub enum RegistryError {
    NetworkError(reqwest::Error),
    AuthError(String),
    ManifestError(String),
    LayerError(String),
    ParseError(String),
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryError::NetworkError(e) => write!(f, "Network error: {}", e),
            RegistryError::AuthError(e) => write!(f, "Authentication error: {}", e),
            RegistryError::ManifestError(e) => write!(f, "Manifest error: {}", e),
            RegistryError::LayerError(e) => write!(f, "Layer error: {}", e),
            RegistryError::ParseError(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for RegistryError {}

impl From<reqwest::Error> for RegistryError {
    fn from(err: reqwest::Error) -> Self {
        RegistryError::NetworkError(err)
    }
}

impl RegistryClient {
    pub fn new(registry_url: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("peeko/0.1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            registry_url: registry_url.trim_end_matches('/').to_string(),
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

    async fn authenticate_if_needed(&mut self, image: &str) -> Result<(), RegistryError> {
        if self.auth_token.is_some() {
            return Ok(());
        }

        // For Docker Hub, try anonymous authentication first
        if self.registry_url.contains("docker.io") {
            self.get_docker_hub_token(image).await?;
            return Ok(());
        }

        let manifest_url = format!("{}/v2/{}/manifests/latest", self.registry_url, image);
        let response = self.client.head(&manifest_url).send().await?;

        if response.status() == 401 {
            if let Some(www_auth) = response.headers().get("www-authenticate") {
                let auth_header = www_auth
                    .to_str()
                    .map_err(|e| RegistryError::AuthError(format!("Invalid auth header: {}", e)))?;

                if auth_header.starts_with("Bearer") {
                    self.get_bearer_token(auth_header, image).await?;
                }
            }
        }

        Ok(())
    }

    async fn get_docker_hub_token(&mut self, image: &str) -> Result<(), RegistryError> {
        let scope = format!("repository:{}:pull", image);
        let token_url = format!(
            "https://auth.docker.io/token?service=registry.docker.io&scope={}",
            scope
        );

        let mut request = self.client.get(&token_url);

        if let (Some(username), Some(password)) = (&self.username, &self.password) {
            request = request.basic_auth(username, Some(password));
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(RegistryError::AuthError(format!(
                "Docker Hub token request failed: {}",
                response.status()
            )));
        }

        let auth_response: AuthToken = response.json().await.map_err(|e| {
            RegistryError::AuthError(format!("Failed to parse Docker Hub auth response: {}", e))
        })?;

        self.auth_token = Some(auth_response.token);
        Ok(())
    }

    async fn get_bearer_token(
        &mut self,
        auth_header: &str,
        image: &str,
    ) -> Result<(), RegistryError> {
        let realm = self
            .extract_auth_param(auth_header, "realm")
            .ok_or_else(|| RegistryError::AuthError("No realm found in auth header".to_string()))?;
        let service = self
            .extract_auth_param(auth_header, "service")
            .unwrap_or_else(|| "registry".to_string());
        let scope = format!("repository:{}:pull", image);

        let token_url = format!("{}?service={}&scope={}", realm, service, scope);

        let mut request = self.client.get(&token_url);

        if let (Some(username), Some(password)) = (&self.username, &self.password) {
            request = request.basic_auth(username, Some(password));
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(RegistryError::AuthError(format!(
                "Token request failed: {}",
                response.status()
            )));
        }

        let auth_response: AuthToken = response.json().await.map_err(|e| {
            RegistryError::AuthError(format!("Failed to parse auth response: {}", e))
        })?;

        self.auth_token = Some(auth_response.token);
        Ok(())
    }

    fn extract_auth_param(&self, auth_header: &str, param: &str) -> Option<String> {
        auth_header.split(',').find_map(|part| {
            let part = part.trim();
            if part.starts_with(&format!("{}=", param)) {
                let value = &part[param.len() + 1..];
                Some(value.trim_matches('"').to_string())
            } else {
                None
            }
        })
    }

    fn add_auth_header(&self, mut request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(token) = &self.auth_token {
            request = request.bearer_auth(token);
        } else if let (Some(username), Some(password)) = (&self.username, &self.password) {
            request = request.basic_auth(username, Some(password));
        }
        request
    }

    pub async fn get_manifest_for_platform(
        &mut self,
        image: &str,
        tag: &str,
        arch: &str,
        os: &str,
    ) -> Result<Manifest, RegistryError> {
        // First try to get the manifest directly (single-arch image)
        self.authenticate_if_needed(image).await?;

        let url = format!("{}/v2/{}/manifests/{}", self.registry_url, image, tag);
        let request = self.client.get(&url)
            .header("Accept", "application/vnd.docker.distribution.manifest.v2+json,application/vnd.oci.image.manifest.v1+json,application/vnd.oci.image.index.v1+json,application/vnd.docker.distribution.manifest.list.v2+json");

        let request = self.add_auth_header(request);
        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(RegistryError::ManifestError(format!(
                "Failed to get manifest: HTTP {}",
                response.status()
            )));
        }

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let response_text = response.text().await?;

        // Check if it's a manifest list/index
        if content_type.contains("manifest.list") || content_type.contains("image.index") {
            let manifest_list: ManifestList =
                serde_json::from_str(&response_text).map_err(|e| {
                    RegistryError::ParseError(format!("Failed to parse manifest list: {}", e))
                })?;

            // Find the manifest for the requested platform
            let target_manifest = manifest_list
                .manifests
                .iter()
                .find(|m| {
                    if let Some(platform) = &m.platform {
                        platform.architecture == arch && platform.os == os
                    } else {
                        false
                    }
                })
                .ok_or_else(|| {
                    RegistryError::ManifestError(format!(
                        "No manifest found for platform {}/{}",
                        os, arch
                    ))
                })?;

            // Get the actual manifest using the digest
            self.get_manifest_by_digest(image, &target_manifest.digest)
                .await
        } else {
            // It's already a single manifest
            let manifest: Manifest = serde_json::from_str(&response_text).map_err(|e| {
                RegistryError::ParseError(format!("Failed to parse manifest: {}", e))
            })?;
            Ok(manifest)
        }
    }

    pub async fn get_manifest(
        &mut self,
        image: &str,
        tag: &str,
    ) -> Result<Manifest, RegistryError> {
        self.get_manifest_for_platform(image, tag, "amd64", "linux")
            .await
    }

    pub async fn get_manifest_list(
        &mut self,
        image: &str,
        tag: &str,
    ) -> Result<ManifestList, RegistryError> {
        self.authenticate_if_needed(image).await?;

        let url = format!("{}/v2/{}/manifests/{}", self.registry_url, image, tag);
        let request = self.client.get(&url)
            .header("Accept", "application/vnd.oci.image.index.v1+json,application/vnd.docker.distribution.manifest.list.v2+json");

        let request = self.add_auth_header(request);
        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(RegistryError::ManifestError(format!(
                "Failed to get manifest list: HTTP {}",
                response.status()
            )));
        }

        let manifest_list: ManifestList = response.json().await.map_err(|e| {
            RegistryError::ParseError(format!("Failed to parse manifest list: {}", e))
        })?;

        Ok(manifest_list)
    }

    pub async fn get_manifest_by_digest(
        &mut self,
        image: &str,
        digest: &str,
    ) -> Result<Manifest, RegistryError> {
        self.authenticate_if_needed(image).await?;

        let url = format!("{}/v2/{}/manifests/{}", self.registry_url, image, digest);
        let request = self.client.get(&url)
            .header("Accept", "application/vnd.docker.distribution.manifest.v2+json,application/vnd.oci.image.manifest.v1+json");

        let request = self.add_auth_header(request);
        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(RegistryError::ManifestError(format!(
                "Failed to get manifest by digest: HTTP {}",
                response.status()
            )));
        }

        let manifest: Manifest = response
            .json()
            .await
            .map_err(|e| RegistryError::ParseError(format!("Failed to parse manifest: {}", e)))?;

        Ok(manifest)
    }

    pub async fn get_manifest_raw(
        &mut self,
        image: &str,
        tag: &str,
    ) -> Result<Value, RegistryError> {
        self.authenticate_if_needed(image).await?;

        let url = format!("{}/v2/{}/manifests/{}", self.registry_url, image, tag);
        let request = self.client.get(&url)
            .header("Accept", "application/vnd.docker.distribution.manifest.v2+json,application/vnd.oci.image.manifest.v1+json");

        let request = self.add_auth_header(request);
        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(RegistryError::ManifestError(format!(
                "Failed to get manifest: HTTP {}",
                response.status()
            )));
        }

        let manifest: Value = response
            .json()
            .await
            .map_err(|e| RegistryError::ParseError(format!("Failed to parse manifest: {}", e)))?;

        Ok(manifest)
    }

    pub async fn download_layer(
        &mut self,
        image: &str,
        digest: &str,
    ) -> Result<Vec<u8>, RegistryError> {
        self.authenticate_if_needed(image).await?;

        let url = format!("{}/v2/{}/blobs/{}", self.registry_url, image, digest);
        let request = self.client.get(&url);
        let request = self.add_auth_header(request);

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(RegistryError::LayerError(format!(
                "Failed to download layer {}: HTTP {}",
                digest,
                response.status()
            )));
        }

        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }

    pub async fn download_layer_stream(
        &mut self,
        image: &str,
        digest: &str,
    ) -> Result<Response, RegistryError> {
        self.authenticate_if_needed(image).await?;

        let url = format!("{}/v2/{}/blobs/{}", self.registry_url, image, digest);
        let request = self.client.get(&url);
        let request = self.add_auth_header(request);

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(RegistryError::LayerError(format!(
                "Failed to download layer {}: HTTP {}",
                digest,
                response.status()
            )));
        }

        Ok(response)
    }

    pub async fn check_layer_exists(
        &mut self,
        image: &str,
        digest: &str,
    ) -> Result<bool, RegistryError> {
        self.authenticate_if_needed(image).await?;

        let url = format!("{}/v2/{}/blobs/{}", self.registry_url, image, digest);
        let request = self.client.head(&url);
        let request = self.add_auth_header(request);

        let response = request.send().await?;
        Ok(response.status().is_success())
    }

    pub async fn get_layer_info(
        &mut self,
        image: &str,
        digest: &str,
    ) -> Result<(u64, String), RegistryError> {
        self.authenticate_if_needed(image).await?;

        let url = format!("{}/v2/{}/blobs/{}", self.registry_url, image, digest);
        let request = self.client.head(&url);
        let request = self.add_auth_header(request);

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(RegistryError::LayerError(format!(
                "Failed to get layer info for {}: HTTP {}",
                digest,
                response.status()
            )));
        }

        let content_length = response
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();

        Ok((content_length, content_type))
    }

    pub async fn list_tags(&mut self, image: &str) -> Result<Vec<String>, RegistryError> {
        self.authenticate_if_needed(image).await?;

        let url = format!("{}/v2/{}/tags/list", self.registry_url, image);
        let request = self.client.get(&url);
        let request = self.add_auth_header(request);

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(RegistryError::ManifestError(format!(
                "Failed to list tags: HTTP {}",
                response.status()
            )));
        }

        let json: Value = response.json().await?;
        let tags = json["tags"]
            .as_array()
            .ok_or_else(|| RegistryError::ParseError("No tags array found".to_string()))?
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();

        Ok(tags)
    }

    pub async fn pull_image(
        &mut self,
        image: &str,
        tag: &str,
    ) -> Result<(Manifest, Vec<Vec<u8>>), RegistryError> {
        let manifest = self.get_manifest(image, tag).await?;
        let mut layers = Vec::new();

        for layer in &manifest.layers {
            println!("Downloading layer: {} ({})", layer.digest, layer.size);
            let layer_data = self.download_layer(image, &layer.digest).await?;

            if layer_data.len() as u64 != layer.size {
                return Err(RegistryError::LayerError(format!(
                    "Layer size mismatch: expected {}, got {}",
                    layer.size,
                    layer_data.len()
                )));
            }

            layers.push(layer_data);
        }

        Ok((manifest, layers))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_manifest() {
        let mut client = RegistryClient::new("https://registry-1.docker.io");

        // Test getting manifest for amd64/linux platform
        let manifest = client
            .get_manifest("library/hello-world", "latest")
            .await
            .unwrap();
        println!("Parsed manifest: {:?}", manifest);
        assert!(manifest.layers.len() > 0);

        // Test getting manifest list
        let manifest_list = client
            .get_manifest_list("library/hello-world", "latest")
            .await
            .unwrap();
        println!(
            "Manifest list has {} entries",
            manifest_list.manifests.len()
        );
        assert!(manifest_list.manifests.len() > 0);
    }

    #[tokio::test]
    async fn test_list_tags() {
        let mut client = RegistryClient::new("https://registry-1.docker.io");
        let tags = client.list_tags("library/hello-world").await.unwrap();
        println!("Tags: {:?}", tags);
        assert!(tags.contains(&"latest".to_string()));
    }

    #[tokio::test]
    async fn test_layer_info() {
        let mut client = RegistryClient::new("https://registry-1.docker.io");
        let manifest = client
            .get_manifest("library/hello-world", "latest")
            .await
            .unwrap();

        if let Some(layer) = manifest.layers.first() {
            let (size, content_type) = client
                .get_layer_info("library/hello-world", &layer.digest)
                .await
                .unwrap();
            println!("Layer size: {}, content-type: {}", size, content_type);
            assert!(size > 0);
        }
    }

    #[tokio::test]
    #[ignore] // 这个测试会下载整个镜像，标记为ignore以避免在CI中运行
    async fn test_pull_image() {
        let mut client = RegistryClient::new("https://registry-1.docker.io");
        let (manifest, layers) = client
            .pull_image("library/hello-world", "latest")
            .await
            .unwrap();

        println!("Pulled {} layers", layers.len());
        assert_eq!(manifest.layers.len(), layers.len());

        for (i, layer) in layers.iter().enumerate() {
            println!("Layer {}: {} bytes", i, layer.len());
        }
    }
}
