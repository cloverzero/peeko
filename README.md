# Peeko

Peeko is a Rust toolkit for exploring container images without launching a container runtime. It ships with:

- **`peeko`** – a library that downloads OCI-compliant images and reconstructs their virtual filesystems.
- **`peeko-cli`** – a command-line interface that lets you pull images, inspect directory trees, and read files directly from the shell.

## Features

**Library (`peeko`)**
- Download image manifests and layers from Docker Hub or any OCI-compatible registry.
- Parse manifests, layer metadata, and build an in-memory virtual filesystem that handles whiteouts and symlinks.
- Read file contents on demand, print directory trees, or collect statistics about image contents.

**CLI (`peeko-cli`)**
- Interactive menu for pulling and browsing images.
- Subcommands for `pull`, `list`, `tree`, `ls`, `cat`, and `remove`.
- Optional progress bars for layer downloads and spinners while building views.

## Installation

### Clone and Build

```bash
git clone <repository-url>
cd peeko

# Build everything
cargo build --release

# Or install just the CLI binary
cargo install --path peeko-cli
```

### Crates.io

```bash
cargo install peeko-cli
```

Add the library to your own project:

```toml
[dependencies]
peeko = "0.1"
# enable download progress bars
peeko = { version = "0.1", features = ["progress"] }
```

## Library Quick Start

```rust
use peeko::{
    reader::build_image_reader,
    registry::{PlatformParam, RegistryClient},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = RegistryClient::new("https://registry-1.docker.io")
        .enable_progress(); // requires the `progress` feature
    let downloads = std::env::temp_dir().join("peeko-downloads");
    client.set_downloads_dir(&downloads);
    client.set_concurrent_downloads(4);

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

    let image_dir = downloads.join("library/alpine/latest");
    let reader = build_image_reader(&image_dir).await?;

    let content = reader.read_file("etc/os-release").await?;
    println!("{}", String::from_utf8_lossy(&content));

    Ok(())
}
```

`Cargo.toml` additions for the snippet:

```toml
[dependencies]
anyhow = "1"
peeko = { version = "0.1", features = ["progress"] }
tokio = { version = "1", features = ["full"] }
```

### Library Helpers

- `peeko::fs::collect_images` enumerates downloaded `image:tag` pairs under a root directory.
- `ImageReader::get_dir_tree` / `print_dir_tree` build tree views for inspection.
- `ImageReader::get_file_meatadata` exposes layer indices and sizes for entries.

## CLI Quick Start

```bash
peeko            # launch the interactive menu
peeko interactive
```

The menu walks you through pulling images, listing cached downloads, browsing trees, and (soon) cleaning up cache directories.

### Common Commands

```bash
peeko pull library/node:18-alpine
peeko pull ghcr.io/owner/app:latest

peeko list

peeko tree library/alpine:latest
peeko tree nginx:latest --path /usr/share/nginx/html --depth 2

peeko ls library/node:18-alpine --path /usr/bin

peeko cat library/alpine:latest --path /etc/os-release

peeko remove library/alpine:latest
```

### CLI Configuration

Environment variables let you customise behaviour:

- `PEEKO_DIR` – directory for cached images (defaults to `~/.peeko`).
- `CONCURRENT_DOWNLOADS` – number of parallel layer downloads (defaults to `4`).

## Repository Layout

```
peeko/       # core library
peeko-cli/   # command-line interface
```

For more detail, see `peeko/README.md` for library APIs and `peeko-cli/README.md` for CLI options.
