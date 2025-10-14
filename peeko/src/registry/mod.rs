//! OCI registry client capable of fetching manifests and downloading layers.

pub mod client;
pub mod progress;

/// Re-export of the high level registry client.
pub use client::{PlatformParam, RegistryClient, RegistryError};
