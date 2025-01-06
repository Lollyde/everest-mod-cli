mod mod_info;
mod download;
mod everest_yaml;

use clap::{Command, Arg, ArgAction};
use mod_info::ModCatalog;
use crate::download::Downloader;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("everest-mod-cli")
        .version("0.1.0")
        .author("Your Name")
        .about("Celeste mod management CLI")
        .subcommand(
            Command::new("search")
                .about("Search for mods")
                .arg(Arg::new("query")
                    .help("Search query")
                    .required(true))
        )
        .subcommand(
            Command::new("info")
                .about("Show mod information")
                .arg(Arg::new("name")
                    .help("Mod name")
                    .required(true))
        )
        .subcommand(
            Command::new("install")
                .about("Install a mod")
                .arg(Arg::new("name")
                    .help("Mod name")
                    .required(true))
        )
        .subcommand(
            Command::new("list")
                .about("List installed mods")
        )
        .subcommand(
            Command::new("show")
                .about("Show detailed information about an installed mod")
                .arg(Arg::new("name")
                    .help("Mod name")
                    .required(true))
        )
        .subcommand(
            Command::new("update")
                .about("Check for updates")
                .arg(Arg::new("install")
                    .long("install")
                    .help("Install available updates")
                    .action(ArgAction::SetTrue))
        )
        .get_matches();

    // Initialize downloader early for list and update commands
    let downloader = Downloader::new()?;

    // Handle list command separately as it doesn't need the catalog
    if let Some(("list", _)) = matches.subcommand() {
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
        return Ok(());
    }

    // Handle show command separately as it only needs installed mods
    if let Some(("show", sub_matches)) = matches.subcommand() {
        let name = sub_matches.get_one::<String>("name").unwrap();
        let installed_mods = downloader.list_installed_mods().await?;
        
        if let Some(mod_info) = installed_mods.iter().find(|m| m.name.as_str() == name) {
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
                println!("No metadata available for mod '{}'", name);
            }
        } else {
            println!("Mod '{}' is not installed", name);
        }
        return Ok(());
    }

    // Load mod catalog for other commands
    let catalog = ModCatalog::fetch_from_network().await?;

    match matches.subcommand() {
        Some(("search", sub_matches)) => {
            let query = sub_matches.get_one::<String>("query").unwrap();
            let results = catalog.search(query);
            
            if results.is_empty() {
                println!("No mods found matching '{}'", query);
            } else {
                println!("Found {} matching mods:", results.len());
                for mod_info in results {
                    println!("\n{} (v{})", mod_info.name, mod_info.version);
                    println!("  Last updated: {}", mod_info.last_update);
                    println!("  URL: {}", mod_info.url);
                }
            }
        }
        Some(("info", sub_matches)) => {
            let name = sub_matches.get_one::<String>("name").unwrap();
            if let Some(mod_info) = catalog.get_mod(name) {
                println!("{} (v{})", mod_info.name, mod_info.version);
                println!("Last updated: {}", mod_info.last_update);
                println!("URL: {}", mod_info.url);
                println!("GameBanana ID: {:?}", mod_info.gamebanana_id);
                println!("Hash: {}", mod_info.hash.join(", "));
            } else {
                println!("Mod '{}' not found", name);
            }
        }
        Some(("install", sub_matches)) => {
            let name = sub_matches.get_one::<String>("name").unwrap();
            
            if let Some(mod_info) = catalog.get_mod(name) {
                let file_path = downloader.download_mod(&mod_info.url, &mod_info.name).await?;

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
                println!("Mod '{}' not found", name);
            }
        }
        Some(("update", sub_matches)) => {
            let install = sub_matches.get_flag("install");
            let updates = downloader.check_updates(&catalog).await?;

            if updates.is_empty() {
                println!("All mods are up to date!");
                return Ok(());
            }

            println!("Available updates:");
            for update in &updates {
                println!("\n{}", update.name);
                println!("  Current version: {}", update.current_version);
                println!("  Available version: {}", update.available_version);
            }

            if install {
                println!("\nInstalling updates...");
                for update in updates {
                    println!("\nUpdating {}...", update.name);
                    let _file_path = downloader.download_mod(&update.url, &update.name).await?;
                    println!("Updated {} to version {}", update.name, update.available_version);
                }
                println!("\nAll updates installed successfully!");
            } else {
                println!("\nRun with --install to install these updates");
            }
        }
        _ => {
            println!("Use --help to see available commands");
        }
    }

    Ok(())
}
