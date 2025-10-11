# Peeko

🐳 **Peeko** is a container image filesystem exploration tool written in Rust, providing a powerful library and command-line interface that allows you to easily browse container image contents without running it.

## Features

### Peeko Core Library (`peeko`)
- **Container Image Pulling**: Support for pulling images from Docker Hub and other OCI-compatible image registries
- **Multi-format Support**: Support for TAR, GZIP, ZSTD and other compressed image layer formats
- **OCI Standard Compliance**: Full support for OCI (Open Container Initiative) image specifications
- **Virtual File System**: Builds a unified virtual filesystem view, handling image layer overlays
- **Multi-platform Support**: Support for pulling and parsing multi-architecture images
- **Concurrent Downloads**: Support for concurrent image layer downloads, improving pull efficiency

### Peeko CLI (`peeko-cli`)
- **Interactive Interface**: User-friendly interactive command-line interface
- **Image Management**: Pull, list, and remove local images
- **Filesystem Browsing**: Browse image filesystem in tree or table format
- **Progress Display**: Real-time download progress display

## Installation

### Build from Source

```bash
# Clone the repository
git clone <repository-url>
cd peeko

# Build the entire project
cargo build --release

# Or build only the CLI tool
cargo build --release -p peeko-cli
```

### Install with Cargo

```bash
cargo install --path peeko-cli
```

## Quick Start

### Interactive Mode (Recommended)

Start interactive mode:

```bash
cargo run
# or
cargo run -- interactive
```

This will display a friendly menu interface:

```
╔══════════════════════════════════════╗
║              PEEKO CLI               ║
║     Container Image Explorer         ║
╚══════════════════════════════════════╝

✨  Welcome to Peeko - the interactive container image explorer!

? What would you like to do? ›
  🐳 Pull new image
  📋 List downloaded images
  🌳 Browse image filesystem
  📊 Show image statistics
  🧹 Clean downloaded images
❯ ❌ Exit
```

### Command Line Mode

#### Pull an Image

```bash
# Pull with specific tag
cargo run -- pull library/node:18-alpine

# Pull from custom registry
cargo run -- pull my-registry.com/library/nginx:latest
```

#### List Downloaded Images

```bash
cargo run -- list
```

#### Browse Filesystem Tree

```bash
# Default view (depth=3, max 10 items per level)
cargo run -- tree library/node:latest

# Custom depth
cargo run -- tree library/node:18-alpine --depth 5

# Browse from specific path
cargo run -- tree library/node:latest --path /usr/bin
```

#### List Directory Contents

```bash
cargo run -- ls library/node:latest --path /usr/bin
```

#### Remove an Image

```bash
cargo run -- remove library/node:18-alpine
```

## Usage Examples

### Example 1: Pull and Explore a Node.js Image

```bash
# Start interactive mode
cargo run

# Select "🐳 Pull new image"
# Enter: library/node
# Tag: 18-alpine
# Registry: (use default)

# Then select "🌳 Browse image filesystem"
# Enter the same image and tag to explore
```

### Example 2: Quick Command Line Usage

```bash
# Pull an image
cargo run -- pull alpine:latest

# Check what was downloaded
cargo run -- list

# Explore the filesystem
cargo run -- tree library/alpine:latest

# View directory contents
cargo run -- ls library/alpine:latest --path /etc
```

## Output Examples

### Filesystem Tree

```
────────────────────────────────────────────────────────
Filesystem Tree for library/alpine:latest
────────────────────────────────────────────────────────
bin/
├── arch
├── ash
├── base64
├── busybox
└── ... and 150 more items
etc/
├── alpine-release
├── apk/
├── group
├── hostname
└── ... and 25 more items
```

