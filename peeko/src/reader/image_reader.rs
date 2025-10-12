use std::io::Read;
use std::path::{Path, PathBuf};

use thiserror::Error;
use tokio::fs;

use super::archive_utils;
use super::dir_tree::DirectoryTree;
use super::vfs::{FileEntry, VirtualFileSystem};
use crate::manifest::{ImageManifest, get_file_type};

#[derive(Error, Debug)]
pub enum ImageReaderError {
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Manifest parse error: {0}")]
    ManifestParseError(#[from] serde_json::Error),

    #[error("Unsupported file type: {0}")]
    UnsupportedFileType(String),

    #[error("File/Directory not found: {0}")]
    NotFound(String),

    #[error("Not a file: {0}")]
    NotAFile(String),
}

pub type Result<T> = std::result::Result<T, ImageReaderError>;

async fn load_manifest<P: AsRef<Path>>(image_dir: P) -> Result<ImageManifest> {
    let manifest_path = image_dir.as_ref().join("manifest.json");
    let manifest = fs::read_to_string(manifest_path).await?;
    let manifest: ImageManifest = serde_json::from_str(&manifest)?;
    Ok(manifest)
}

async fn load_layer<P: AsRef<Path>>(
    layer_path: P,
    file_type: &str,
    layer_index: usize,
    vfs: &mut VirtualFileSystem,
) -> Result<()> {
    let mut archive = match file_type {
        "tar" => archive_utils::read_tar_file(layer_path)?,
        "gzip" => archive_utils::read_gzip_file(layer_path)?,
        "zstd" => archive_utils::read_zstd_file(layer_path)?,
        _ => return Err(ImageReaderError::UnsupportedFileType(file_type.to_string())),
    };

    for entry in archive.entries()? {
        let entry = entry?;
        let path = entry.path()?.to_path_buf();
        let header = entry.header();

        // 处理 whiteout 文件
        if let Some(filename) = path.file_name() {
            let filename_str = filename.to_string_lossy();

            if filename_str.starts_with(".wh.") {
                if filename_str == ".wh..wh..opq" {
                    // 删除整个目录内容
                    if let Some(parent) = path.parent() {
                        println!("  Clearing directory: {:?}", parent);
                        vfs.clear_directory(parent);
                    }
                } else {
                    // 删除特定文件
                    let target_name = filename_str.strip_prefix(".wh.").unwrap();
                    if let Some(parent) = path.parent() {
                        let target_path = parent.join(target_name);
                        println!("  Removing (whiteout): {:?}", target_path);
                        vfs.delete_entry(&target_path);
                    }
                }
                continue;
            }
        }

        match header.entry_type() {
            tar::EntryType::Regular => vfs.add_entry(
                path,
                FileEntry::File {
                    size: entry.size(),
                    layer_index,
                },
            ),
            tar::EntryType::Directory => vfs.add_entry(path, FileEntry::Directory { layer_index }),
            tar::EntryType::Symlink | tar::EntryType::Link => {
                if let Ok(link_name) = header.link_name() {
                    if let Some(link_name) = link_name {
                        vfs.add_entry(
                            path,
                            FileEntry::Symlink {
                                target: link_name.to_string_lossy().to_string(),
                                layer_index,
                            },
                        );
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

async fn read_file_from_layer<LP: AsRef<Path>, FP: AsRef<Path>>(
    layer_path: LP,
    file_type: &str,
    file_path: FP,
) -> Result<String> {
    let layer_path = layer_path.as_ref();
    let file_path = file_path.as_ref();

    let mut archive = match file_type {
        "tar" => archive_utils::read_tar_file(layer_path)?,
        "gzip" => archive_utils::read_gzip_file(layer_path)?,
        "zstd" => archive_utils::read_zstd_file(layer_path)?,
        _ => return Err(ImageReaderError::UnsupportedFileType(file_type.to_string())),
    };

    let target = archive.entries()?.into_iter().find(|entry| match entry {
        Ok(entry) => entry.path().map_or(false, |path| path.eq(file_path)),
        Err(_) => false,
    });

    match target {
        Some(Ok(mut entry)) => {
            let mut buf = String::new();
            match entry.read_to_string(&mut buf) {
                Ok(_) => Ok(buf),
                Err(err) => Err(err.into()),
            }
        }
        Some(Err(err)) => Err(err.into()),
        None => Err(ImageReaderError::NotFound(
            file_path.to_string_lossy().to_string(),
        )),
    }
}

pub async fn build_image_reader<P: AsRef<Path>>(image_dir: P) -> Result<ImageReader> {
    let image_dir = image_dir.as_ref();
    let manifest = load_manifest(image_dir).await?;

    let mut vfs = VirtualFileSystem::new();
    for (layer_index, layer) in manifest.layers.iter().enumerate() {
        let file_type = get_file_type(&layer.media_type);
        let layer_path = image_dir.join(format!("{}.{}", layer.digest, file_type));
        load_layer(layer_path, file_type, layer_index, &mut vfs).await?;
    }

    Ok(ImageReader {
        image_dir: image_dir.to_path_buf(),
        manifest,
        vfs,
    })
}

pub struct ImageReader {
    image_dir: PathBuf,
    manifest: ImageManifest,
    vfs: VirtualFileSystem,
}

impl ImageReader {
    pub async fn read_file<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        let path = path.as_ref();
        let entry = self
            .vfs
            .get_entry(path)
            .ok_or_else(|| ImageReaderError::NotFound(path.to_string_lossy().to_string()))?;
        if let FileEntry::File { layer_index, .. } = entry {
            let layer = &self.manifest.layers[*layer_index];
            let file_type = get_file_type(&layer.media_type);
            let layer_path = self
                .image_dir
                .join(format!("{}.{}", layer.digest, file_type));
            let content = read_file_from_layer(&layer_path, file_type, path).await?;
            Ok(content)
        } else {
            Err(ImageReaderError::NotAFile(
                path.to_string_lossy().to_string(),
            ))
        }
    }

    pub fn get_dir_tree(&self) -> Result<DirectoryTree> {
        let tree = self.vfs.get_directory_tree();
        Ok(tree)
    }

    pub fn print_dir_tree(&self, depth: usize, path: Option<String>) -> Result<()> {
        let tree = self.get_dir_tree()?;
        let target_node = match &path {
            Some(path) => tree.find(path),
            None => Some(tree.root),
        };

        match target_node {
            Some(node) => {
                node.print(0, depth, true, "");
                Ok(())
            }
            None => Err(ImageReaderError::NotFound(
                path.unwrap_or_else(|| "/".to_string()),
            )),
        }
    }

    pub fn get_file_meatadata(&self, path: &str) -> Option<&FileEntry> {
        self.vfs.get_entry(&PathBuf::from(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_reconstruct() {
        let r = build_image_reader("library/node/24-alpine").await;
        assert!(r.is_ok());
    }
}
