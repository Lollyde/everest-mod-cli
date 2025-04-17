use bytes::Bytes;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::path::{Path, PathBuf};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
};
use xxhash_rust::xxh64::Xxh64;

use crate::everest_yaml::{ModMetadata, ModMetadataList};
use crate::mod_info::ModCatalog;

pub const MOD_REGISTRY_URL: &str = "https://maddie480.ovh/celeste/everest_update.yaml";

#[derive(Debug)]
pub struct UpdateInfo {
    pub name: String,
    pub current_version: String,
    pub available_version: String,
    pub url: String,
}

#[derive(Debug)]
pub struct InstalledMod {
    pub name: String,
    pub metadata: Option<ModMetadata>,
}

pub struct Downloader {
    client: Client,
    download_dir: PathBuf,
}

impl Downloader {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let home = std::env::var("HOME").map_err(|_| "Could not determine home directory")?;
        let download_dir =
            PathBuf::from(home).join(".local/share/Steam/steamapps/common/Celeste/Mods");

        if !download_dir.exists() {
            return Err(
                "Celeste mods directory not found. Is Celeste installed through Steam?".into(),
            );
        }

        Ok(Self {
            client: Client::new(),
            download_dir,
        })
    }

    pub async fn fetch_mod_registry(&self) -> Result<Bytes, reqwest::Error> {
        let response = self.client.get(MOD_REGISTRY_URL).send().await?;
        let yaml_data = response.bytes().await?;
        Ok(yaml_data)
    }

    pub async fn check_updates(
        &self,
        catalog: &ModCatalog,
    ) -> Result<Vec<UpdateInfo>, Box<dyn std::error::Error>> {
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
                        });
                    }
                }
            }
        }

        Ok(updates)
    }

    pub async fn download_mod(
        &self,
        url: &str,
        name: &str,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let response = self.client.get(url).send().await?;
        let total_size = response.content_length().unwrap_or(0);

        let pb = ProgressBar::new(total_size);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("#>-"));

        let mut stream = response.bytes_stream();
        let download_path = self.download_dir.join(format!("{}.zip", name));
        let mut file = fs::File::create(&download_path).await?;
        let mut downloaded: u64 = 0;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            let new = std::cmp::min(downloaded + (chunk.len() as u64), total_size);
            downloaded = new;
            pb.set_position(new);
        }

        pb.finish_with_message("Download complete");

        // Verify checksum if available
        if let Some(mod_info) = ModCatalog::fetch_from_network().await?.get_mod(name) {
            if !mod_info.hash.is_empty() {
                let expected_hash = &mod_info.hash[0];
                if verify_checksum(&download_path, expected_hash).await? {
                    println!("Checksum verification successful");
                } else {
                    fs::remove_file(&download_path).await?;
                    return Err("Checksum verification failed".into());
                }
            }
        }

        Ok(download_path)
    }

    pub async fn list_installed_mods(
        &self,
    ) -> Result<Vec<InstalledMod>, Box<dyn std::error::Error>> {
        let mut installed_mods = Vec::new();

        // Create directory if it doesn't exist
        if !self.download_dir.exists() {
            return Ok(installed_mods);
        }

        let mut entries = fs::read_dir(&self.download_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("zip") {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                // Try to read everest.yaml from the zip
                let mod_metadata = match ModMetadataList::from_zip(&path) {
                    Ok(list) => list.get_main_mod().cloned(),
                    Err(e) => {
                        eprintln!(
                            "Warning: Failed to read metadata from {}: {}",
                            path.display(),
                            e
                        );
                        None
                    }
                };

                installed_mods.push(InstalledMod {
                    name,
                    metadata: mod_metadata,
                });
            }
        }

        // Sort by name
        installed_mods.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(installed_mods)
    }
}

pub async fn hash_file(file_path: &Path) -> std::io::Result<String> {
    let file = fs::File::open(file_path).await?;
    let mut reader = BufReader::new(file);

    let mut hasher = Xxh64::new(0);
    let mut buffer = [0u8; 8192]; // Read in 8 KB chunks

    loop {
        let bytes_read = reader.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:016x}", hasher.digest()))
}

pub async fn verify_checksum(file_path: &Path, expected_hash: &str) -> std::io::Result<bool> {
    let file = fs::File::open(file_path).await?;
    let mut reader = BufReader::new(file);

    let mut hasher = Xxh64::new(0);
    let mut buffer = [0u8; 8192]; // Read in 8 KB chunks

    loop {
        let bytes_read = reader.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let hash = format!("{:016x}", hasher.digest());

    println!("Computed hash: {}", hash);
    println!("Expected hash: {}", expected_hash);

    Ok(hash == expected_hash.to_lowercase())
}

fn compare_versions(ver1: &str, ver2: &str) -> std::cmp::Ordering {
    let v1_parts: Vec<&str> = ver1.split('.').collect();
    let v2_parts: Vec<&str> = ver2.split('.').collect();

    for i in 0..std::cmp::max(v1_parts.len(), v2_parts.len()) {
        let n1 = v1_parts
            .get(i)
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        let n2 = v2_parts
            .get(i)
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);

        match n1.cmp(&n2) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }
    std::cmp::Ordering::Equal
}
