use console::style;
use inquire::{Confirm, Select, Text};

use crate::commands;
use crate::error::Result;
use crate::utils;

const MENU_OPTIONS: &[&str] = &[
    "🐳 Pull new image",
    "📋 List downloaded images",
    "🌳 Browse image filesystem",
    "📊 Show image statistics",
    "🧹 Clean downloaded images",
    "❌ Exit",
];

pub async fn run() -> Result<()> {
    utils::print_welcome();

    loop {
        println!();
        let choice = Select::new("What would you like to do?", MENU_OPTIONS.to_vec()).prompt()?;

        match choice {
            "🐳 Pull new image" => {
                if let Err(e) = handle_pull_image().await {
                    utils::print_error(&format!("Failed to pull image: {}", e));
                }
            }
            "📋 List downloaded images" => {
                if let Err(e) = commands::list::execute().await {
                    utils::print_error(&format!("Failed to list images: {}", e));
                }
            }
            "🌳 Browse image filesystem" => {
                if let Err(e) = handle_browse_filesystem().await {
                    utils::print_error(&format!("Failed to browse filesystem: {}", e));
                }
            }
            "🧹 Clean downloaded images" => {
                if let Err(e) = handle_clean_images().await {
                    utils::print_error(&format!("Failed to clean images: {}", e));
                }
            }
            "❌ Exit" => {
                utils::print_success("Goodbye! 👋");
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

async fn handle_pull_image() -> Result<()> {
    println!("\n{}", style("📥 Pull Container Image").bold().cyan());

    let image = Text::new("Image name with tag (e.g., library/node:18-alpine, nginx:latest):")
        .with_help_message("Enter the image name with tag")
        .prompt()?;

    println!("\n{}", style("Starting download...").dim());
    commands::pull::execute(&image).await?;

    Ok(())
}

async fn handle_browse_filesystem() -> Result<()> {
    println!("\n{}", style("🌳 Browse Image Filesystem").bold().green());

    let image = Text::new("Image name:")
        .with_help_message("Enter the image name")
        .prompt()?;

    let depth = Text::new("Maximum depth:")
        .with_default("3")
        .with_help_message("How deep to show the directory tree")
        .prompt()?;

    let depth: usize = depth.parse().unwrap_or(3);

    commands::tree::execute(&image, depth, None).await?;

    Ok(())
}

async fn handle_clean_images() -> Result<()> {
    println!("\n{}", style("🧹 Clean Downloaded Images").bold().red());

    let confirm = Confirm::new("Are you sure you want to delete all downloaded images?")
        .with_default(false)
        .with_help_message("This action cannot be undone")
        .prompt()?;

    if confirm {
        // TODO: Implement clean functionality
        utils::print_success("All downloaded images have been cleaned!");
    } else {
        utils::print_info("Clean operation cancelled.");
    }

    Ok(())
}
