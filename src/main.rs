use clap::Parser;

mod cli;
mod download;
mod everest_yaml;
mod mod_info;

use cli::{Cli, Commands};
use download::Downloader;
use mod_info::ModCatalog;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Initialize downloader early for list and update commands
    let downloader = Downloader::new()?;

    match &cli.command {
        Commands::List => {
            let installed_mods = downloader.list_installed_mods().await?;
            if installed_mods.is_empty() {
                println!("No mods installed");
                return Ok(());
            }

            println!("Installed mods:");
            for mod_info in installed_mods {
                if let Some(metadata) = mod_info.metadata {
                    println!("  {} v{}", mod_info.name, metadata.version);
                } else {
                    println!("  {} (no metadata)", mod_info.name);
                }
            }
        }

        Commands::Show(args) => {
            let installed_mods = downloader.list_installed_mods().await?;
            if let Some(mod_info) = installed_mods.iter().find(|m| m.name == args.name) {
                if let Some(metadata) = &mod_info.metadata {
                    println!("Name: {}", mod_info.name);
                    println!("Version: {}", metadata.version);

                    if let Some(deps) = &metadata.dependencies {
                        println!("\nDependencies:");
                        for dep in deps {
                            if let Some(ver) = &dep.version {
                                println!("  - {} v{}", dep.name, ver);
                            } else {
                                println!("  - {}", dep.name);
                            }
                        }
                    }

                    if let Some(opt_deps) = &metadata.optional_dependencies {
                        println!("\nOptional Dependencies:");
                        for dep in opt_deps {
                            if let Some(ver) = &dep.version {
                                println!("  - {} v{}", dep.name, ver);
                            } else {
                                println!("  - {}", dep.name);
                            }
                        }
                    }
                } else {
                    println!("No metadata available for mod '{}'", args.name);
                }
            } else {
                println!("Mod '{}' is not installed", args.name);
            }
        }

        // For remaining commands, load the mod catalog from the network
        _ => {
            let mod_registry = downloader.fetch_mod_registry().await?;
            let catalog = ModCatalog::new(mod_registry).await?;

            match &cli.command {
                Commands::Search(args) => {
                    let results = catalog.search(&args.query);
                    if results.is_empty() {
                        println!("No mods found matching '{}'", args.query);
                    } else {
                        println!("Found {} matching mods:", results.len());
                        for mod_info in results {
                            println!("\n{} (v{})", mod_info.name, mod_info.version);
                            println!("  Last updated: {}", mod_info.last_update);
                            println!("  URL: {}", mod_info.url);
                        }
                    }
                }
                Commands::Info(args) => {
                    if let Some(mod_info) = catalog.get_mod(&args.name) {
                        println!("{} (v{})", mod_info.name, mod_info.version);
                        println!("Last updated: {}", mod_info.last_update);
                        println!("URL: {}", mod_info.url);
                        println!("GameBanana ID: {:?}", mod_info.gamebanana_id);
                        println!("Hash: {}", mod_info.hash.join(", "));
                    } else {
                        println!("Mod '{}' not found", args.name);
                    }
                }
                Commands::Install(args) => {
                    if let Some(mod_info) = catalog.get_mod(&args.name) {
                        let file_path = downloader
                            .download_mod(&mod_info.url, &mod_info.name)
                            .await?;
                        if !mod_info.hash.is_empty() {
                            println!("Verifying download...");
                            let hash = &mod_info.hash[0];
                            if download::verify_checksum(&file_path, hash).await? {
                                println!("Checksum verification successful!");
                            } else {
                                println!("Error: Checksum verification failed!");
                                tokio::fs::remove_file(file_path).await?;
                                return Err("Checksum verification failed".into());
                            }
                        } else {
                            println!("Warning: No checksum available for verification");
                        }
                    } else {
                        println!("Mod '{}' not found", args.name);
                    }
                }
                Commands::Update(args) => {
                    let updates = downloader.check_updates(&catalog).await?;
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
                                let _file_path =
                                    downloader.download_mod(&update.url, &update.name).await?;
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
