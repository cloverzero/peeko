use std::fs;
use std::io::Result;
use std::path::{Path, PathBuf};

pub fn collect_images<P: AsRef<Path>>(oci_dir: P) -> Result<Vec<String>> {
    let base_dir = oci_dir.as_ref();
    collect_image_directories(base_dir).map(|dirs| {
        dirs.into_iter()
            .map(|dir| {
                let mut relative_path = dir
                    .strip_prefix(base_dir)
                    .expect("Must be a subdirectory of the peeko directory")
                    .to_string_lossy()
                    .to_string();
                if let Some(pos) = relative_path.rfind('/') {
                    relative_path.replace_range(pos..pos + 1, ":")
                }

                relative_path
            })
            .collect()
    })
}

pub fn collect_image_directories<P: AsRef<Path>>(path: P) -> Result<Vec<PathBuf>> {
    let mut result = Vec::new();
    let path = path.as_ref();
    collect_image_directories_recursive(path, &mut result)?;
    Ok(result)
}

fn collect_image_directories_recursive(path: &Path, result: &mut Vec<PathBuf>) -> Result<()> {
    if !path.is_dir() {
        return Ok(());
    }

    let manifest_path = path.join("manifest.json");
    if manifest_path.exists() {
        result.push(path.to_path_buf());
        return Ok(());
    }

    let entries = fs::read_dir(path)?;
    for entry in entries
        .flatten()
        .filter(|e| e.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
    {
        collect_image_directories_recursive(&entry.path(), result)?;
    }

    Ok(())
}

pub fn delete_image<P: AsRef<Path>>(oci_dir: P, image: &str, tag: &str) -> Result<()> {
    let image_path = oci_dir.as_ref().join(format!("{image}/{tag}"));
    fs::remove_dir_all(&image_path)?;
    Ok(())
}
