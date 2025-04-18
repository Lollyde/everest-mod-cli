use clap::Parser;

mod cli;
mod constant;
mod download;
mod error;
mod fileutil;
mod installed_mods;
mod mod_registry;

use cli::{Cli, Commands};
use download::ModDownloader;
use mod_registry::ModRegistry;
use tracing::{debug, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .compact()
        .with_max_level(tracing::Level::ERROR)
        .with_file(false)
        .with_line_number(false)
        .with_thread_ids(false)
        .with_target(false)
        .without_time()
        .init();

    info!("Application starts");

    let cli = Cli::parse();
    debug!("Command passed: {:#?}", &cli.command);

    // Initialize downloader early for list and update commands
    let downloader = ModDownloader::new();

    match &cli.command {
        Commands::List => {
            let installed_mods = downloader.list_installed_mods()?;
            if installed_mods.is_empty() {
                println!("No mods installed");
                return Ok(());
            }

            println!("Installed mods ({}):", installed_mods.len());
            for mod_info in installed_mods {
                println!("    {} v{}", mod_info.mod_name, mod_info.version);
            }
        }

        Commands::Show(args) => {
            let installed_mods = downloader.list_installed_mods()?;
            if let Some(mod_info) = installed_mods.iter().find(|m| m.mod_name == args.name) {
                println!("Name: {}", mod_info.mod_name);
                println!("Version: {}", mod_info.version);
            } else {
                println!("Mod '{}' is not installed", args.name);
            }
        }

        // For remaining commands, fetch the remote mod registry
        _ => {
            let mod_registry_data = downloader.fetch_mod_registry().await?;
            let mod_registry = ModRegistry::from(mod_registry_data).await?;

            match &cli.command {
                Commands::Search(args) => {
                    let results = mod_registry.search(&args.query);
                    if results.is_empty() {
                        println!("No mods found matching '{}'", args.query);
                    } else {
                        println!("Found {} matching mods:", results.len());
                        for mod_info in results {
                            println!("\n{} (v{})", mod_info.name, mod_info.version);
                            println!("  Last updated: {}", mod_info.updated_at);
                            println!("  URL: {}", mod_info.download_url);
                        }
                    }
                }
                Commands::Info(args) => {
                    if let Some(mod_info) = mod_registry.get_mod_info(&args.name) {
                        println!("{} (v{})", mod_info.name, mod_info.version);
                        println!("Last updated: {}", mod_info.updated_at);
                        println!("Download link: {}", mod_info.download_url);
                        println!(
                            "Page URL: https://gamebanana.com/mods/{}",
                            mod_info.gamebanana_id
                        );
                        println!("Hashes: {}", mod_info.checksums.join(", "));
                    } else {
                        println!("Mod '{}' not found", args.name);
                    }
                }
                Commands::Install(args) => {
                    if let Some(mod_info) = mod_registry.get_mod_info(&args.name) {
                        downloader
                            .download_mod(
                                &mod_info.download_url,
                                &mod_info.name,
                                &mod_info.checksums,
                            )
                            .await?;
                    } else {
                        println!("Mod '{}' not found", args.name);
                    }
                }
                Commands::Update(args) => {
                    let updates = downloader.check_updates(&mod_registry).await?;
                    if updates.is_empty() {
                        println!("All mods are up to date!");
                    } else {
                        println!("Available updates:");
                        for update in &updates {
                            println!("\n{}", update.name);
                            println!("  Current version: {}", update.current_version);
                            println!("  Available version: {}", update.available_version);
                        }
                        if args.install {
                            println!("\nInstalling updates...");
                            for update in updates {
                                println!("\nUpdating {}...", update.name);
                                downloader
                                    .download_mod(&update.url, &update.name, &update.hash)
                                    .await?;
                                println!(
                                    "Updated {} to version {}",
                                    update.name, update.available_version
                                );
                            }
                            println!("\nAll updates installed successfully!");
                        } else {
                            println!("\nRun with --install to install these updates");
                        }
                    }
                }
                // Catch-all arm (should not be reached because all subcommands are handled)
                _ => {
                    println!("Use --help to see available commands");
                }
            }
        }
    }

    Ok(())
}
