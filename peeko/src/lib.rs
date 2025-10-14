//! Core library for interacting with OCI container images that have been
//! downloaded to disk. The crate provides helpers for discovering images on the
//! filesystem, parsing image manifests, reading layer contents, downloading
//! artifacts from registries, and computing simple statistics about virtual
//! filesystems reconstructed from image layers.

/// Filesystem helpers for working with OCI image layouts stored on disk.
pub mod fs;
/// Types that model OCI image manifests and configs.
pub mod manifest;
/// Async readers that reconstruct a virtual filesystem view of image layers.
pub mod reader;
/// Clients for talking to OCI compatible registries.
pub mod registry;
/// Utilities for summarising reconstructed filesystem trees.
pub mod stats;
