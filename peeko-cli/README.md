# Peeko CLI

Peeko CLI is a command-line companion to the Peeko library. It lets you pull OCI container images, inspect their filesystem without running them, and manage your local cache of previously downloaded layers.

## Prerequisites

- Rust toolchain with Cargo (the workspace targets the 2024 edition)
- Access to an OCI-compatible registry (Docker Hub by default)

## Installation

From a cloned workspace:

```bash
cargo install --path peeko-cli
# or build locally:
cargo build --release -p peeko-cli
```

If `peeko-cli` is published to crates.io you can also run:

```bash
cargo install peeko-cli
```

## Configuration

Peeko CLI stores data under `~/.peeko` by default. You can override the defaults with environment variables:

- `PEEKO_DIR` – base directory for downloaded images
- `CONCURRENT_DOWNLOADS` – number of parallel layer downloads (defaults to `4`)

## Usage Overview

```
peeko <COMMAND> [OPTIONS]
```

Run `peeko -h` or `peeko <COMMAND> -h` for full clap-generated help.

### Interactive Mode

```bash
peeko            # starts the interactive menu
peeko interactive
```

The interactive menu guides you through pulling images, listing cached images, browsing a filesystem tree, and (soon) cleaning up downloads.

## Commands

### Pull

```bash
peeko pull library/node:18-alpine
peeko pull ghcr.io/owner/app:latest
```

- Fetches the image manifest and layers into `PEEKO_DIR`
- Docker Hub is used when the registry is omitted (`library/` is prefixed automatically)
- Fails fast if you forget the `:tag`

### List

```bash
peeko list
```

Shows a table of cached images and their on-disk size, e.g.:

```
Downloaded Images
Image            Tag           Size
library/alpine   latest        5.3 MB

Found 1 downloaded image(s)
```

### Tree

```bash
peeko tree library/alpine:latest
peeko tree library/node:18-alpine --depth 5
peeko tree nginx:latest --path /usr/share/nginx/html --depth 2
```

- Prints a formatted filesystem tree rendered from the merged layers
- Requires the image to have been pulled already
- `--depth` controls recursion (default `3`)
- `--path` lets you explore a subtree

### Ls

```bash
peeko ls library/alpine:latest --path /
peeko ls library/node:18-alpine --path /usr/bin
```

- Displays the contents of a directory as a table (`Type`, `Size`, `File`)
- Path must be provided with `--path`

### Cat

```bash
peeko cat library/alpine:latest --path /etc/os-release
```

- Streams file contents to stdout
- Accepts absolute or relative paths (leading `/` is optional)

### Remove

```bash
peeko remove library/alpine:latest
```

Deletes the cached image directory inside `PEEKO_DIR`. This does not interact with the remote registry.

## Tips

- Pull the image before running `tree`, `ls`, or `cat`; they operate on the local cache.
- For large images, increase `CONCURRENT_DOWNLOADS` to improve throughput.
- Combine with standard shell tools (e.g. pipe `peeko cat` into `grep`) to inspect files quickly.
