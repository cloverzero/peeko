use std::path::{Path, PathBuf};

use tokio::fs;

use super::archive_utils;
use super::vfs::{FileEntry, VirtualFileSystem};
use crate::manifest::{ImageManifest, get_file_type};

pub struct ImageReader {
    image_dir: PathBuf,
    manifest: Option<ImageManifest>,
    vfs: Option<VirtualFileSystem>,
}

impl ImageReader {
    pub fn new<P: AsRef<Path>>(image_dir: P) -> Self {
        Self {
            image_dir: image_dir.as_ref().to_path_buf(),
            manifest: None,
            vfs: None,
        }
    }

    async fn load_manifest(&self) -> anyhow::Result<ImageManifest> {
        let manifest_path = self.image_dir.join("manifest.json");
        let manifest = fs::read_to_string(manifest_path).await?;
        let manifest: ImageManifest = serde_json::from_str(&manifest)?;
        Ok(manifest)
    }

    pub async fn reconstruct(&mut self) -> anyhow::Result<()> {
        let manifest = self.load_manifest().await?;
        let mut vfs = VirtualFileSystem::new();
        for (layer_index, layer) in manifest.layers.iter().enumerate() {
            let file_type = get_file_type(&layer.media_type);
            let layer_path = self
                .image_dir
                .join(format!("{}.{}", layer.digest, file_type));
            self.load_layer(layer_path, file_type, layer_index, &mut vfs)
                .await?;
        }

        self.manifest = Some(manifest);
        self.vfs = Some(vfs);

        Ok(())
    }

    async fn load_layer(
        &mut self,
        layer_path: PathBuf,
        file_type: &str,
        layer_index: usize,
        vfs: &mut VirtualFileSystem,
    ) -> anyhow::Result<()> {
        let mut archive = match file_type {
            "tar" => archive_utils::read_tar_file(layer_path)?,
            "gzip" => archive_utils::read_gzip_file(layer_path)?,
            "zstd" => archive_utils::read_zstd_file(layer_path)?,
            _ => return Err(anyhow::anyhow!("Unsupported file type: {}", file_type)),
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
                tar::EntryType::Directory => {
                    vfs.add_entry(path, FileEntry::Directory { layer_index })
                }
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

    pub fn print_dir_tree(&self, depth: usize) {
        let tree = self.vfs.as_ref().unwrap().get_directory_tree(Some(depth));
        tree.print(depth);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_reconstruct() {
        let mut reader = ImageReader::new("library/node/24-alpine");
        let r = reader.reconstruct().await;
        assert!(r.is_ok());
    }
}
