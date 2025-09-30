use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "mediaType")]
pub enum Manifest {
    #[serde(rename = "application/vnd.oci.image.manifest.v1+json")]
    OCIManifest(ImageManifest),

    #[serde(rename = "application/vnd.oci.image.index.v1+json")]
    OCIIndex(ManifestList),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ImageManifest {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub config: Descriptor,
    pub layers: Vec<Descriptor>,

    // for oci index
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Descriptor {
    pub digest: String,
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub size: u64,

    // for oci index
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ManifestList {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub manifests: Vec<PlatformManifest>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlatformManifest {
    pub digest: String,
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub platform: Platform,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Platform {
    pub architecture: String,
    pub os: String,
    #[serde(rename = "os.version", skip_serializing_if = "Option::is_none")]
    pub os_version: Option<String>,
    #[serde(rename = "os.features", skip_serializing_if = "Option::is_none")]
    pub os_features: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
}

pub fn get_file_type(media_type: &str) -> &str {
    match media_type.rsplit_once('+') {
        Some((_, ext)) => ext,
        None => match media_type.rsplit_once('.') {
            Some((_, ext)) => ext,
            None => "tar",
        },
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageConfig {
    pub architecture: String,
    pub os: String,
    pub config: ContainerConfig,
    pub created: String,
    pub history: Vec<HistoryEntry>,
    pub rootfs: RootFs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerConfig {
    #[serde(rename = "Hostname")]
    pub hostname: Option<String>,

    #[serde(rename = "User")]
    pub user: Option<String>,

    #[serde(rename = "Env")]
    pub env: Option<Vec<String>>,

    #[serde(rename = "Cmd")]
    pub cmd: Option<Vec<String>>,

    #[serde(rename = "Entrypoint")]
    pub entrypoint: Option<Vec<String>>,

    #[serde(rename = "WorkingDir")]
    pub working_dir: Option<String>,

    #[serde(rename = "Labels")]
    pub labels: Option<HashMap<String, String>>,

    #[serde(rename = "ExposedPorts")]
    pub exposed_ports: Option<HashMap<String, serde_json::Value>>,

    #[serde(rename = "Volumes")]
    pub volumes: Option<HashMap<String, serde_json::Value>>,

    #[serde(rename = "StopSignal")]
    pub stop_signal: Option<String>,

    #[serde(rename = "Shell")]
    pub shell: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub created: String,
    pub created_by: String,

    #[serde(default)]
    pub empty_layer: bool,

    #[serde(default)]
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootFs {
    #[serde(rename = "type")]
    pub fs_type: String,
    pub diff_ids: Vec<String>,
}
