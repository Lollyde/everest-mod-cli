#![allow(deprecated)]
use std::{
    env::home_dir,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

use zip::{ZipArchive, result::ZipError};

use crate::constant::{MOD_MANIFEST_FILE, STEAM_MODS_DIRECTORY_PATH};
use crate::error::Error;

/// Returns the path to the user's mods directory based on platform-specific conventions
pub fn get_mods_directory() -> PathBuf {
    // NOTE: `std::env::home_dir()` will be undeprecated in rust 1.87.0
    home_dir()
        .map(|home_path| home_path.join(STEAM_MODS_DIRECTORY_PATH))
        .expect("Unable to determine home directory location!")
}

/// Scans the mods directory and returns a list of all installed mod archive files (.zip)
pub fn find_installed_mod_archives(mods_directory: &Path) -> Result<Vec<PathBuf>, Error> {
    // Verify the mods directory exists
    if let Err(err) = fs::read_dir(mods_directory) {
        if err.kind() == std::io::ErrorKind::NotFound {
            return Err(Error::MissingModsDirectory);
        }
    }

    // Collect all .zip files in the directory
    let mod_archives = fs::read_dir(mods_directory)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| path.extension().is_some_and(|ext| ext == "zip"))
        .collect();

    Ok(mod_archives)
}

/// Reads the mod manifest file from a given ZIP archive.
pub fn read_manifest_file_from_zip(zip_path: &Path) -> Result<Option<String>, Error> {
    let zip_file = File::open(zip_path)?;
    let mut zip_archive = ZipArchive::new(zip_file)?;

    match zip_archive.by_name(MOD_MANIFEST_FILE) {
        Ok(mut file) => {
            let mut content = String::new();
            file.read_to_string(&mut content)?;

            // Remove potential BOM if exists
            if content.starts_with('\u{feff}') {
                content.remove(0);
            }
            Ok(Some(content))
        }
        Err(ZipError::FileNotFound) => Ok(None),
        Err(err) => Err(Error::Io(err.into())),
    }
}
