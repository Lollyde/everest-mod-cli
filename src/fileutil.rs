#![allow(deprecated)]
use std::{
    env::home_dir,
    fs::{self, File},
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

use tracing::info;
use xxhash_rust::xxh64::Xxh64;
use zip::{ZipArchive, result::ZipError};

use crate::constant::{MOD_MANIFEST_FILE, STEAM_MODS_DIRECTORY_PATH};
use crate::error::Error;

/// Returns the path to the user's mods directory based on platform-specific conventions
pub fn get_mods_directory() -> Result<PathBuf, Error> {
    info!("Detecting Celeste/Mods directory...");
    // NOTE: `std::env::home_dir()` will be undeprecated in rust 1.87.0
    home_dir()
        .map(|home_path| home_path.join(STEAM_MODS_DIRECTORY_PATH))
        .ok_or(Error::CouldNotDetermineHomeDir)
}

/// Scans the mods directory and returns a list of all installed mod archive files (.zip)
pub fn find_installed_mod_archives(mods_directory: &Path) -> Result<Vec<PathBuf>, Error> {
    if !mods_directory.exists() {
        return Err(Error::MissingModsDirectory);
    }

    info!("Scanning installed mod archives in {:?}", mods_directory);

    let mut mod_archives = Vec::new();
    let entries = fs::read_dir(mods_directory)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "zip") {
            mod_archives.push(path);
        }
    }

    Ok(mod_archives)
}

/// Reads the mod manifest file from a given ZIP archive.
pub fn read_manifest_file_from_zip(zip_path: &Path) -> Result<Option<Vec<u8>>, Error> {
    let zip_file = File::open(zip_path)?;
    let reader = BufReader::new(zip_file);
    let mut zip_archive = ZipArchive::new(reader)?;

    match zip_archive.by_name(MOD_MANIFEST_FILE) {
        Ok(mut file) => {
            // NOTE: Max file size of `everest.yaml` should be under 10KB
            let mut buffer = Vec::with_capacity(12 * 1024);
            file.read_to_end(&mut buffer)?;

            // Check for UTF-8 BOM and remove if present
            if buffer.len() >= 3 && buffer[0] == 0xEF && buffer[1] == 0xBB && buffer[2] == 0xBF {
                buffer.drain(0..3);
            }

            Ok(Some(buffer))
        }
        Err(ZipError::FileNotFound) => Ok(None),
        Err(err) => Err(Error::Io(err.into())),
    }
}

/// Compute xxhash of a given file, return hexadicimal string
pub fn hash_file(file_path: &Path) -> Result<String, Error> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;
    use tempfile::NamedTempFile;
    use zip::write::{SimpleFileOptions, ZipWriter};

    const MOD_MANIFEST_FILE: &str = "everest.yaml";

    // Helper function to create a zip file with a manifest
    fn create_test_zip(manifest_content: Option<&[u8]>) -> NamedTempFile {
        let temp_file = NamedTempFile::new().unwrap();
        let file = File::create(temp_file.path()).unwrap();
        let mut zip = ZipWriter::new(file);

        if let Some(content) = manifest_content {
            zip.start_file(MOD_MANIFEST_FILE, SimpleFileOptions::default())
                .unwrap();
            zip.write_all(content).unwrap();
        }

        zip.finish().unwrap();
        temp_file
    }

    #[test]
    fn test_read_manifest_file_success() {
        let content = b"test manifest content".to_vec();
        let temp_zip = create_test_zip(Some(&content));

        let result = read_manifest_file_from_zip(temp_zip.path());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(content));
    }

    #[test]
    fn test_read_manifest_file_with_utf8_bom() {
        let mut content = vec![0xEF, 0xBB, 0xBF]; // UTF-8 BOM
        content.extend_from_slice(b"test manifest content");
        let expected_content = b"test manifest content".to_vec();
        let temp_zip = create_test_zip(Some(&content));

        let result = read_manifest_file_from_zip(temp_zip.path());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(expected_content));
    }

    #[test]
    fn test_read_manifest_file_not_found() {
        let temp_zip = create_test_zip(None);

        let result = read_manifest_file_from_zip(temp_zip.path());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_read_invalid_zip_file() {
        let temp_file = NamedTempFile::new().unwrap();
        File::create(temp_file.path())
            .unwrap()
            .write_all(b"not a zip file")
            .unwrap();

        let result = read_manifest_file_from_zip(temp_file.path());

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::Zip(ZipError::InvalidArchive(_))
        ));
    }

    #[test]
    fn test_read_nonexistent_file() {
        let nonexistent_path = Path::new("nonexistent.zip");

        let result = read_manifest_file_from_zip(nonexistent_path);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Io(_)));
    }
}
