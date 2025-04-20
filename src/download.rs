use bytes::Bytes;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::{
    io::Read,
    path::{Path, PathBuf},
    time::Instant,
};
use tokio::{fs, io::AsyncWriteExt};
use tracing::{info, warn};
use xxhash_rust::xxh64::Xxh64;

use crate::{
    constant::MOD_REGISTRY_URL,
    error::Error,
    fileutil::{find_installed_mod_archives, read_manifest_file_from_zip},
    installed_mods::{InstalledModList, LocalModInfo, ModManifest},
    mod_registry::ModRegistry,
};

/// Update information about the mod
#[derive(Debug)]
pub struct AvailableUpdateInfo {
    /// The Mod name
    pub name: String,
    /// Current version (from LocalModInfo)
    pub current_version: String,
    /// Available version (from RemoteModInfo)
    pub available_version: String,
    /// Download URL of the Mod
    pub url: String,
    /// xxHashes of the file
    pub hash: Vec<String>,
    /// Outdated file
    pub existing_path: PathBuf,
}

/// Manage mod downloads
#[derive(Debug, Clone)]
pub struct ModDownloader {
    client: Client,
    registry_url: String,
    download_dir: PathBuf,
}

impl ModDownloader {
    pub fn new(download_dir: &Path) -> Self {
        Self {
            client: Client::new(),
            registry_url: String::from(MOD_REGISTRY_URL),
            download_dir: download_dir.to_path_buf(),
        }
    }

    /// Fetch remote mod registry, returns bytes of response
    pub async fn fetch_mod_registry(&self) -> Result<Bytes, Error> {
        info!("Fetching remote mod registry...");
        let response = self.client.get(&self.registry_url).send().await?;
        let yaml_data = response.bytes().await?;
        Ok(yaml_data)
    }

    // Check available updates for all installed mods
    pub fn check_updates(
        &self,
        mod_registry: &ModRegistry,
    ) -> Result<Vec<AvailableUpdateInfo>, Error> {
        let installed_mods = self.list_installed_mods()?;
        let mut available_updates = Vec::new();

        for local_mod in installed_mods {
            if let Some(remote_mod) = mod_registry.get_mod_info(&local_mod.mod_name) {
                if remote_mod.has_matching_hash(&local_mod.checksum) {
                    continue; // No update avilable
                };
                let available_mod = remote_mod.clone();
                available_updates.push(AvailableUpdateInfo {
                    name: local_mod.mod_name,
                    current_version: local_mod.version,
                    available_version: available_mod.version,
                    url: available_mod.download_url,
                    hash: available_mod.checksums,
                    existing_path: local_mod.archive_path,
                });
            }
        }

        Ok(available_updates)
    }

    /// Download mod file and verify checksum
    pub async fn download_mod(
        &self,
        url: &str,
        name: &str,
        expected_hash: &[String],
    ) -> Result<(), Error> {
        info!("Start downloading mod: {}", name);

        let response = self.client.get(url).send().await?.error_for_status()?;
        info!("Status code: {}", response.status().as_u16());

        let filename = util::determine_filename(&response)?;
        let download_path = self.download_dir.join(format!("{}.zip", filename));
        info!("Destination: {}", download_path.display());

        let total_size = response.content_length().unwrap_or(0);
        info!("Total file size: {}", total_size);

        let pb = ProgressBar::new(total_size);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("#>-"));

        let mut stream = response.bytes_stream();

        let mut hasher = Xxh64::new(0);
        let mut file = fs::File::create(&download_path).await?;
        let mut downloaded: u64 = 0;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            hasher.update(&chunk);
            let new = std::cmp::min(downloaded + (chunk.len() as u64), total_size);
            downloaded = new;
            pb.set_position(new);
        }

        pb.finish_with_message("Download complete");

        // Verify checksum
        let hash = hasher.digest();
        let hash_str = format!("{:016x}", hash);
        info!("xxhash of downloaded file: {}", hash_str);

        println!("\n  Verifying checksum...");
        if expected_hash.contains(&hash_str) {
            println!("  Checksum verified!");
        } else {
            println!("  Checksum verification failed!");
            fs::remove_file(&download_path).await?;
            println!("  Downloaded file removed");
            return Err(Error::InvalidChecksum {
                file: download_path,
                computed: hash_str,
                expected: expected_hash.to_vec(),
            });
        }

        Ok(())
    }

    /// List installed mods which has valid manifest file
    pub fn list_installed_mods(&self) -> Result<InstalledModList, Error> {
        info!(
            "Collecting information about installed mods... This might take a few minutes if your mods library is huge"
        );
        let start = Instant::now();

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
                None => {
                    let debug_path = archive_path
                        .file_name()
                        .and_then(|path| path.to_str())
                        .expect("File name shoud be exist");
                    warn!(
                        "No mod manifest file (everest.yaml) found in {}.\n\
                    \t# The file might be named 'everest.yml' or located in a subdirectory.\n\
                    \t# Please contact the mod creator about this issue or just ignore this message.\n\
                    \t# Updates will be skipped for this mod.",
                        debug_path
                    )
                }
            }
        }

        // Sort by name
        info!("Sorting results by name...");
        installed_mods.sort_by(|a, b| a.mod_name.cmp(&b.mod_name));

        let duration = start.elapsed();
        info!("Finished collecting in: {:#?}", duration);

        Ok(installed_mods)
    }
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

mod util {
    use super::*;
    use reqwest::{Response, Url};
    use uuid::Uuid;

    /// Determines the most appropriate filename for a downloaded mod using URL and metadata
    pub fn determine_filename(response: &Response) -> Result<String, Error> {
        // Try to extract filename from the URL path.
        let filename_from_url = extract_filename_from_url(response.url());

        // Try to extract filename from the ETag header.
        let filename_from_etag = extract_filename_from_etag(response);

        // Choose the best available filename or generate a random one
        let mod_filename = filename_from_url
            .or(filename_from_etag)
            .unwrap_or_else(|| format!("unknown-mod_{}.zip", Uuid::new_v4()));

        Ok(mod_filename)
    }

    /// Extracts a filename from the last segment of a URL path
    fn extract_filename_from_url(url: &Url) -> Option<String> {
        url.path_segments()
            .and_then(|mut segments| segments.next_back().filter(|&segment| !segment.is_empty()))
            .map(String::from)
    }

    /// Creates a filename using the ETag header value, properly formatted with extension
    fn extract_filename_from_etag(response: &Response) -> Option<String> {
        response
            .headers()
            .get(reqwest::header::ETAG)
            .and_then(|etag_value| etag_value.to_str().ok())
            .map(|etag| etag.trim_matches('"').to_string())
            .map(|etag| format!("{}.zip", etag))
    }
}
