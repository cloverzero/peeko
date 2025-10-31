# Peeko Library

Peeko is a Rust library for fetching OCI container images and reconstructing their filesystems without starting a container runtime. It offers two high-level building blocks:

- `registry::RegistryClient` – downloads image manifests and layer blobs into a local OCI-style directory layout.
- `reader::build_image_reader` – builds an in-memory virtual filesystem so you can inspect files, directories, and metadata from the downloaded image.

## Installation

Add the crate to your project:

```toml
[dependencies]
peeko = { path = "../peeko" }          # workspace use
# peeko = "0.1"                        # when published
```

Optional progress indicators require the `progress` feature:

```toml
peeko = { version = "0.1", features = ["progress"] }
```

This pulls in `indicatif` to show download progress bars when you call `RegistryClient::enable_progress`.

## End-to-End Example

The following async example downloads `library/alpine:latest`, reads `/etc/os-release` from the reconstructed filesystem, and prints it. It stores artifacts under a temporary directory.

```rust
use peeko::{
    reader::build_image_reader,
    registry::{PlatformParam, RegistryClient},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Configure the registry client
    let mut client = RegistryClient::new("https://registry-1.docker.io")
        .enable_progress(); // requires the `progress` feature
    let downloads = std::env::temp_dir().join("peeko-downloads");
    client.set_downloads_dir(&downloads);
    client.set_concurrent_downloads(4);

    // 2. Pull the image (platform filters are optional)
    client
        .download_image(
            "library/alpine",
            "latest",
            PlatformParam {
                architecture: None,
                os: None,
                variant: None,
            },
        )
        .await?;

    // 3. Build a reader from the downloaded image directory
    let image_dir = downloads.join("library/alpine/latest");
    let reader = build_image_reader(&image_dir).await?;

    // 4. Read a file from the virtual filesystem
    let content = reader.read_file("etc/os-release").await?;
    println!("{}", String::from_utf8_lossy(&content));

    Ok(())
}
```

Example `Cargo.toml` additions for the snippet above:

```toml
[dependencies]
anyhow = "1"
peeko = { version = "0.1", features = ["progress"] }
tokio = { version = "1", features = ["full"] }
```

### What Happens Behind the Scenes

1. `RegistryClient::download_image` fetches the manifest for the requested tag, resolves the correct platform from a manifest list (if necessary), and writes `manifest.json` plus all layer blobs (named `<digest>.<ext>`) into the downloads directory.
2. `build_image_reader` replays the layers in order, handling whiteouts and symlinks to produce an in-memory virtual filesystem.
3. `ImageReader::read_file` streams the requested file from the layer blob that last wrote it, so you see the final merged view.

## Additional Helpers

- `peeko::fs::collect_images` scans a root directory (such as `~/.peeko`) and returns `image:tag` identifiers for everything downloaded.
- `ImageReader::get_dir_tree` and `print_dir_tree` generate recursive directory listings.
- `ImageReader::get_file_meatadata` exposes the backing layer index and size for each entry.

With these building blocks you can create custom tooling—for example, enforce policy on image contents, extract configuration files, or generate inventory reports—without needing Docker installed.
