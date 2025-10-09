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
        /// Maximum depth to show
        #[arg(short, long, default_value = "3")]
        depth: usize,
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
        Some(Commands::Tree { image, depth }) => {
            commands::tree::execute(&image, depth).await?;
        }
        Some(Commands::Interactive) | None => {
            interactive::run().await?;
        }
    }

    Ok(())
}
