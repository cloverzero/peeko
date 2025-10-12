use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod interactive;
mod utils;

#[derive(Parser)]
#[command(name = "peeko")]
#[command(about = "Container image filesystem explorer")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Pull a container image from registry
    Pull {
        /// Image name (e.g., library/node, nginx)
        image: String,
    },
    /// List downloaded images
    List,
    /// Remove a downloaded image
    Remove {
        /// Image name with tag (e.g., library/node:18-alpine, nginx:latest)
        image: String,
    },
    /// Show image filesystem tree
    Tree {
        /// Image name with tag (e.g., library/node:18-alpine, nginx:latest)
        image: String,

        /// Path to start the tree from
        #[arg(short, long)]
        path: Option<String>,

        /// Maximum depth to show
        #[arg(short, long, default_value = "3")]
        depth: usize,
    },
    /// List files in an image
    Ls {
        /// Image name with tag (e.g., library/node:18-alpine, nginx:latest)
        image: String,

        /// Path to start the ls from
        #[arg(short, long)]
        path: String,
    },
    /// Cat a file in an image
    Cat {
        /// Image name with tag (e.g., library/node:18-alpine, nginx:latest)
        image: String,

        /// Path to the file to cat
        #[arg(short, long)]
        path: String,
    },
    /// Start interactive mode
    Interactive,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Pull { image }) => {
            commands::pull::execute(&image).await?;
        }
        Some(Commands::List) => {
            commands::list::execute().await?;
        }
        Some(Commands::Remove { image }) => {
            commands::remove::execute(&image).await?;
        }
        Some(Commands::Tree { image, depth, path }) => {
            commands::tree::execute(&image, depth, path).await?;
        }
        Some(Commands::Ls { image, path }) => {
            commands::ls::execute(&image, &path).await?;
        }
        Some(Commands::Cat { image, path }) => {
            commands::cat::execute(&image, &path).await?;
        }
        Some(Commands::Interactive) | None => {
            interactive::run().await?;
        }
    }

    Ok(())
}
