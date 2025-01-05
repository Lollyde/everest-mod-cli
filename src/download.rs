use std::path::{Path, PathBuf};
use reqwest::Client;
use tokio::fs;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use dir::home_dir;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use crate::everest_yaml::{ModMetadataList, ModMetadata};
use crate::mod_info::ModCatalog;

#[derive(Debug)]
pub struct UpdateInfo {
    pub name: String,
    pub current_version: String,
    pub available_version: String,
    pub url: String,
    pub file_size: u64,
}

pub struct ModDownloader {
    client: Client,
    download_dir: PathBuf,
}

#[derive(Debug)]
pub struct InstalledMod {
    pub name: String,
    pub file_size: u64,
    pub file_path: PathBuf,
    pub metadata: Option<ModMetadata>,
}

impl ModDownloader {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mods_dir = get_mods_directory()?;
        Ok(Self {
            client: Client::new(),
            download_dir: mods_dir,
        })
    }

    pub async fn check_updates(&self, catalog: &ModCatalog) -> Result<Vec<UpdateInfo>, Box<dyn std::error::Error>> {
        let installed_mods = self.list_installed_mods().await?;
        let mut updates = Vec::new();

        for installed_mod in installed_mods {
            if let Some(metadata) = installed_mod.metadata {
                if let Some(available_mod) = catalog.get_mod(&metadata.name) {
                    // Compare versions
                    if compare_versions(&available_mod.version, &metadata.version).is_gt() {
                        updates.push(UpdateInfo {
                            name: metadata.name,
                            current_version: metadata.version,
                            available_version: available_mod.version.clone(),
                            url: available_mod.url.clone(),
                            file_size: installed_mod.file_size,
                        });
                    }

                    // Check dependencies for updates
                    if let Some(deps) = metadata.dependencies {
                        for dep in deps {
                            if let Some(available_dep) = catalog.get_mod(&dep.name) {
                                if let Some(current_ver) = dep.version {
                                    if compare_versions(&available_dep.version, &current_ver).is_gt() {
                                        updates.push(UpdateInfo {
                                            name: dep.name,
                                            current_version: current_ver,
                                            available_version: available_dep.version.clone(),
                                            url: available_dep.url.clone(),
                                            file_size: 0, // Unknown until downloaded
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(updates)
    }

    pub async fn download_mod(&self, url: &str, name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        // Create mods directory if it doesn't exist
        fs::create_dir_all(&self.download_dir).await?;

        // Start the download
        println!("Downloading {} from {}", name, url);
        let response = self.client.get(url).send().await?;
        
        // Get content length for progress bar
        let total_size = response
            .content_length()
            .unwrap_or(0);

        // Setup progress bar
        let pb = ProgressBar::new(total_size);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("#>-"));

        // Get the file name from the URL or use the mod name
        let file_name = format!("{}.zip", name);
        let file_path = self.download_dir.join(&file_name);

        // Stream the download to file with progress
        let mut file = fs::File::create(&file_path).await?;
        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();

        while let Some(item) = stream.next().await {
            let chunk = item?;
            file.write_all(&chunk).await?;
            downloaded = downloaded.saturating_add(chunk.len() as u64);
            pb.set_position(downloaded);
        }

        pb.finish_with_message(format!("Downloaded {} successfully", name));
        Ok(file_path)
    }

    pub async fn list_installed_mods(&self) -> Result<Vec<InstalledMod>, Box<dyn std::error::Error>> {
        let mut installed_mods = Vec::new();

        // Create directory if it doesn't exist
        if !self.download_dir.exists() {
            return Ok(installed_mods);
        }

        let mut entries = fs::read_dir(&self.download_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("zip") {
                let metadata = entry.metadata().await?;
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                // Try to read everest.yaml from the zip
                let mod_metadata = match ModMetadataList::from_zip(&path) {
                    Ok(list) => list.get_main_mod().cloned(),
                    Err(e) => {
                        eprintln!("Warning: Failed to read metadata from {}: {}", path.display(), e);
                        None
                    }
                };

                installed_mods.push(InstalledMod {
                    name,
                    file_size: metadata.len(),
                    file_path: path,
                    metadata: mod_metadata,
                });
            }
        }

        // Sort by name
        installed_mods.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(installed_mods)
    }
}

pub fn get_mods_directory() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = home_dir().ok_or("Could not find home directory")?;
    Ok(home.join(".local/share/Steam/steamapps/common/Celeste/Mods"))
}

pub async fn verify_checksum(file_path: &Path, expected_hash: &str) -> Result<bool, Box<dyn std::error::Error>> {
    use sha1::{Sha1, Digest};
    use tokio::io::AsyncReadExt;

    let mut file = fs::File::open(file_path).await?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await?;

    let mut hasher = Sha1::new();
    hasher.update(&buffer);
    let result = hasher.finalize();
    let hash = hex::encode(result);

    Ok(hash == expected_hash)
}

pub fn format_size(size: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}

fn compare_versions(ver1: &str, ver2: &str) -> std::cmp::Ordering {
    let v1_parts: Vec<&str> = ver1.split('.').collect();
    let v2_parts: Vec<&str> = ver2.split('.').collect();

    for i in 0..std::cmp::max(v1_parts.len(), v2_parts.len()) {
        let n1 = v1_parts.get(i).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
        let n2 = v2_parts.get(i).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
        
        match n1.cmp(&n2) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }
    std::cmp::Ordering::Equal
}
