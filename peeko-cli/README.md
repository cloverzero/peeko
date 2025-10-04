# Peeko CLI

An interactive command-line tool for exploring container image filesystems.

## Features

- 🐳 Pull container images from registries
- 📋 List downloaded images
- 🌳 Browse image filesystem tree
- 📊 Show image statistics
- 💻 Interactive mode with user-friendly prompts

## Installation

```bash
cargo build --release
```

## Usage

### Interactive Mode (Recommended)

Start the interactive mode by running without arguments:

```bash
cargo run
# or
cargo run -- interactive
```

This will present you with a user-friendly menu to:
- Pull new images
- Browse downloaded images
- Explore filesystem trees
- View statistics

### Command Line Mode

#### Pull an image

```bash
# Pull with default tag (latest)
cargo run -- pull library/node

# Pull with specific tag
cargo run -- pull library/node --tag 18-alpine

# Pull from custom registry
cargo run -- pull nginx --tag latest --registry https://my-registry.com
```

#### List downloaded images

```bash
cargo run -- list
```

#### Browse filesystem tree

```bash
# Default view (depth=3, max 10 items per level)
cargo run -- tree library/node

# Custom depth and item limits
cargo run -- tree library/node --tag 18-alpine --depth 5 --max-items 15
```

#### Show image statistics

```bash
cargo run -- stats library/node --tag 18-alpine
```

## Examples

### Example 1: Pull and explore a Node.js image

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

### Example 2: Quick command-line usage

```bash
# Pull an image
cargo run -- pull library/alpine --tag latest

# Check what was downloaded
cargo run -- list

# Explore the filesystem
cargo run -- tree library/alpine

# View statistics
cargo run -- stats library/alpine
```

## Output Examples

### Interactive Menu
```
╔══════════════════════════════════════╗
║            🐳 PEEKO CLI             ║
║     Container Image Explorer        ║
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

### Statistics
```
=== Filesystem Statistics ===
Total directories: 45
Total files: 189
Total symlinks: 12
Total size: 7.85 MB
```