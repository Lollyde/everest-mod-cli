use bytes::Bytes;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::{
    io::Read,
    path::{Path, PathBuf},
};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
};
use xxhash_rust::xxh64::Xxh64;

use crate::{
    constant::MOD_REGISTRY_URL,
    error::Error,
    fileutil::{find_installed_mod_archives, get_mods_directory, read_manifest_file_from_zip},
    installed_mods::{InstalledModList, LocalModInfo, ModManifest},
    mod_registry::ModRegistry,
};

#[derive(Debug)]
pub struct UpdateInfo {
    pub name: String,
    pub current_version: String,
    pub available_version: String,
    pub url: String,
    pub hash: Vec<String>,
}

/// Manage mod downloads
#[derive(Debug, Clone)]
pub struct ModDownloader {
    client: Client,
    registry_url: String,
    download_dir: PathBuf,
}

impl ModDownloader {
    pub fn new() -> Self {
        let download_dir = get_mods_directory();

        Self {
            client: Client::new(),
            registry_url: String::from(MOD_REGISTRY_URL),
            download_dir,
        }
    }

    /// Fetch remote mod registry, returns bytes of response
    pub async fn fetch_mod_registry(&self) -> Result<Bytes, Error> {
        println!("Fetching remote mod registry...");
        let response = self.client.get(&self.registry_url).send().await?;
        let yaml_data = response.bytes().await?;
        Ok(yaml_data)
    }

    // TODO: change logic to hash comparison
    pub async fn check_updates(
        &self,
        mod_registry: &ModRegistry,
    ) -> Result<Vec<UpdateInfo>, Box<dyn std::error::Error>> {
        let installed_mods = self.list_installed_mods()?;
        let mut updates = Vec::new();

        for installed_mod in installed_mods {
            if let Some(available_mod) = mod_registry.get_mod_info(&installed_mod.mod_name) {
                // Compare versions
                if compare_versions(&available_mod.version, &installed_mod.version).is_gt() {
                    updates.push(UpdateInfo {
                        name: installed_mod.mod_name,
                        current_version: installed_mod.version,
                        available_version: available_mod.version.clone(),
                        url: available_mod.download_url.clone(),
                        hash: available_mod.checksums.clone(),
                    });
                }
            }
        }

        Ok(updates)
    }

    /// Download mod file and verify checksum
    pub async fn download_mod(
        &self,
        url: &str,
        name: &str,
        expected_hash: &[String],
    ) -> Result<(), Error> {
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

        // Verify checksum
        let hash = async_hash_file(&download_path).await?;
        if expected_hash.contains(&hash) {
            println!("Checksum verified");
        } else {
            fs::remove_file(&download_path).await?;
            return Err(Error::InvalidChecksum {
                file: download_path,
                computed: hash,
                expected: expected_hash.to_vec(),
            });
        }

        Ok(())
    }

    /// List installed mods which has valid manifest file
    pub fn list_installed_mods(&self) -> Result<InstalledModList, Error> {
        let archive_paths = find_installed_mod_archives(&self.download_dir)?;
        let mut installed_mods = Vec::with_capacity(archive_paths.len());

        for archive_path in archive_paths {
            let manifest_content = read_manifest_file_from_zip(&archive_path)?;
            match manifest_content {
                Some(content) => {
                    let checksum = sync_hash_file(&archive_path)?;
                    let manifest = ModManifest::parse_mod_manifest_from_yaml(&content)?;
                    let local_mod = LocalModInfo::new(archive_path, manifest, checksum);
                    installed_mods.push(local_mod);
                }
                None => println!(
                    "No mod manifest file (everest.yaml) found in {}.\n\
                        #  The file might be named 'everest.yml' or located in a subdirectory.\n\
                        # Please contact the mod creator about this issue.\n\
                        # Updates will be skipped for this mod.\n",
                    archive_path.display()
                ),
            }
        }

        // Sort by name
        installed_mods.sort_by(|a, b| a.mod_name.cmp(&b.mod_name));
        Ok(installed_mods)
    }
}

/// Compute xxhash of a given file, return hexadicimal string (async version)
pub async fn async_hash_file(file_path: &Path) -> Result<String, Error> {
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
    let hash_str = format!("{:016x}", hasher.digest());
    Ok(hash_str)
}

/// Compute xxhash of a given file, return hexadicimal string (sync version)
pub fn sync_hash_file(file_path: &Path) -> Result<String, Error> {
    let file = std::fs::File::open(file_path)?;
    let mut reader = std::io::BufReader::new(file);
    let mut hasher = Xxh64::new(0);
    let mut buffer = [0u8; 8192]; // Read in 8 KB chunks
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    let hash_str = format!("{:016x}", hasher.digest());
    Ok(hash_str)
}

// TODO: make this hash comparison
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
