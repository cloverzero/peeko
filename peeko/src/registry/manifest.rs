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
