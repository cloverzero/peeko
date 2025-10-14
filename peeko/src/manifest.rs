use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// High level representation of OCI manifest documents.
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "mediaType")]
pub enum Manifest {
    #[serde(rename = "application/vnd.oci.image.manifest.v1+json")]
    OCIManifest(ImageManifest),

    #[serde(rename = "application/vnd.oci.image.index.v1+json")]
    OCIIndex(ManifestList),
}

/// Representation of `application/vnd.oci.image.manifest.v1+json`.
#[derive(Debug, Deserialize, Serialize)]
pub struct ImageManifest {
    #[serde(rename = "schemaVersion")]
    /// Schema version declared by the manifest.
    pub schema_version: u32,
    #[serde(rename = "mediaType")]
    /// Media type for the manifest.
    pub media_type: String,
    /// Descriptor that points to the configuration blob.
    pub config: Descriptor,
    /// Ordered layer descriptors composing the image.
    pub layers: Vec<Descriptor>,

    // for oci index
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Optional annotations supplied by the image registry.
    pub annotations: Option<HashMap<String, String>>,
}

/// Generic descriptor that points to a blob stored in the registry.
#[derive(Debug, Deserialize, Serialize)]
pub struct Descriptor {
    /// SHA digest of the referenced blob.
    pub digest: String,
    #[serde(rename = "mediaType")]
    /// Media type of the blob.
    pub media_type: String,
    /// Size in bytes of the blob.
    pub size: u64,

    // for oci index
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Additional annotations provided by the registry.
    pub annotations: Option<HashMap<String, String>>,
}

/// Representation of `application/vnd.oci.image.index.v1+json`.
#[derive(Debug, Deserialize, Serialize)]
pub struct ManifestList {
    #[serde(rename = "schemaVersion")]
    /// Schema version declared by the manifest.
    pub schema_version: u32,
    #[serde(rename = "mediaType")]
    /// Media type for the manifest list.
    pub media_type: String,
    /// Architectures and platforms included in the manifest list.
    pub manifests: Vec<PlatformManifest>,
}

/// Descriptor of a single platform entry inside an OCI index.
#[derive(Debug, Deserialize, Serialize)]
pub struct PlatformManifest {
    /// SHA digest of the platform-specific manifest.
    pub digest: String,
    #[serde(rename = "mediaType")]
    /// Media type for the manifest.
    pub media_type: String,
    /// Target platform described by this manifest.
    pub platform: Platform,
    /// Size in bytes of the manifest blob.
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Optional annotations supplied by the registry.
    pub annotations: Option<HashMap<String, String>>,
}

/// Platform information attached to a platform manifest descriptor.
#[derive(Debug, Deserialize, Serialize)]
pub struct Platform {
    /// CPU architecture (for example `amd64` or `arm64`).
    pub architecture: String,
    /// Operating system (for example `linux`).
    pub os: String,
    #[serde(rename = "os.version", skip_serializing_if = "Option::is_none")]
    /// Optional OS version.
    pub os_version: Option<String>,
    #[serde(rename = "os.features", skip_serializing_if = "Option::is_none")]
    /// Optional OS feature list.
    pub os_features: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// CPU variant (for example `v7`).
    pub variant: Option<String>,
}

/// Returns the file extension associated with a descriptor's media type.
pub fn get_file_type(media_type: &str) -> &str {
    match media_type.rsplit_once('+') {
        Some((_, ext)) => ext,
        None => match media_type.rsplit_once('.') {
            Some((_, ext)) => ext,
            None => "tar",
        },
    }
}

/// Runtime configuration extracted from an image config blob.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageConfig {
    /// CPU architecture (for example `amd64` or `arm64`).
    pub architecture: String,
    /// Operating system (for example `linux`).
    pub os: String,
    /// Container runtime settings.
    pub config: ContainerConfig,
    /// Timestamp when the image was created.
    pub created: String,
    /// History describing how the image layers were produced.
    pub history: Vec<HistoryEntry>,
    /// Root filesystem diff IDs.
    pub rootfs: RootFs,
}

/// Container runtime options section inside an image config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerConfig {
    #[serde(rename = "Hostname")]
    /// Default hostname assigned to containers started from the image.
    pub hostname: Option<String>,

    #[serde(rename = "User")]
    /// Default user (UID/GID) the container should run as.
    pub user: Option<String>,

    #[serde(rename = "Env")]
    /// Default environment variables.
    pub env: Option<Vec<String>>,

    #[serde(rename = "Cmd")]
    /// Default command executed by the container runtime.
    pub cmd: Option<Vec<String>>,

    #[serde(rename = "Entrypoint")]
    /// Entrypoint process invoked before `Cmd`.
    pub entrypoint: Option<Vec<String>>,

    #[serde(rename = "WorkingDir")]
    /// Working directory for the default command.
    pub working_dir: Option<String>,

    #[serde(rename = "Labels")]
    /// Image labels attached to the runtime config.
    pub labels: Option<HashMap<String, String>>,

    #[serde(rename = "ExposedPorts")]
    /// Ports exposed by default.
    pub exposed_ports: Option<HashMap<String, serde_json::Value>>,

    #[serde(rename = "Volumes")]
    /// Named volumes declared by the image.
    pub volumes: Option<HashMap<String, serde_json::Value>>,

    #[serde(rename = "StopSignal")]
    /// Signal used to request graceful shutdown.
    pub stop_signal: Option<String>,

    #[serde(rename = "Shell")]
    /// Default shell used for command interpretation.
    pub shell: Option<Vec<String>>,
}

/// Detailed history line for how an image layer was produced.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Timestamp when the layer was created.
    pub created: String,
    /// Command that produced the layer.
    pub created_by: String,

    #[serde(default)]
    /// Whether the entry represents an empty layer.
    pub empty_layer: bool,

    #[serde(default)]
    /// Optional comment attached to the history entry.
    pub comment: Option<String>,
}

/// Root filesystem metadata inside an image config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootFs {
    #[serde(rename = "type")]
    /// Type of filesystem (typically `layers`).
    pub fs_type: String,
    /// Digest list representing layer diff IDs.
    pub diff_ids: Vec<String>,
}
