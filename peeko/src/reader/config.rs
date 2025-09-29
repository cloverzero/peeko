use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

impl ImageConfig {
    pub fn from_str(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}
