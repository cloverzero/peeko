//! Helpers for reconstructing filesystem content from OCI image layers.

mod archive_utils;
mod dir_tree;
mod image_reader;
pub mod vfs;

/// Error type returned by the asynchronous image reader.
pub use image_reader::ImageReaderError;
/// Build a high level image reader from an unpacked OCI image directory.
pub use image_reader::build_image_reader;
