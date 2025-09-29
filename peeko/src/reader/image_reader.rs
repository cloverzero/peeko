use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use tokio::fs;

use crate::reader::config::ImageConfig;

use super::vfs::VirtualFileSystem;

pub struct ImageReader {
    image_dir: PathBuf,
    fs: VirtualFileSystem,
}

impl ImageReader {
    pub fn new<P: AsRef<Path>>(image_dir: P) -> Self {
        Self {
            image_dir: image_dir.as_ref().to_path_buf(),
            fs: VirtualFileSystem::new(),
        }
    }

    async fn load_dir(&self) -> anyhow::Result<HashMap<String, String>> {
        let mut file_map = HashMap::new();
        let mut entries = fs::read_dir(&self.image_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let metadata = entry.metadata().await?;
            if metadata.is_file() {
                let file_name = entry.file_name();
                let file_name = file_name.to_string_lossy();
                if let Some((name, ext)) = file_name.rsplit_once('.') {
                    file_map.insert(name.to_string(), ext.to_string());
                }
            }
        }
        Ok(file_map)
    }

    async fn load_config(
        &self,
        file_map: &HashMap<String, String>,
    ) -> anyhow::Result<Option<ImageConfig>> {
        let config_file = file_map.iter().find(|(_, ext)| *ext == "json");
        if let Some((name, _)) = config_file {
            let config_path = self.image_dir.join(format!("{}.json", name));
            let json = fs::read_to_string(config_path).await?;
            let image_config: ImageConfig = ImageConfig::from_str(&json)?;
            return Ok(Some(image_config));
        }

        Ok(None)
    }

    pub async fn reconstruct(&mut self) -> anyhow::Result<()> {
        let file_map = self.load_dir().await?;
        let config = self.load_config(&file_map).await?;
        let config = config.ok_or_else(|| anyhow::anyhow!("No config found"))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_reconstruct() {
        let mut reader = ImageReader::new("library/hello-world/latest");
        reader.reconstruct().await.unwrap();
    }
}
