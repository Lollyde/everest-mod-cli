# Everest Mod CLI

**WIP**. This project is currently under development. Use at your own risk.

A command-line interface tool for managing Celeste mods using Everest API.

Currently, target **Steam** and **Linux** installation. Flatpak version is not supported.

## Issues

- [x] `ModCatalog::fetch_from_network()` is being called twice in some operations
- [ ] Version comparison is not meaningful. The value might contain a nonsensical string instead of a number. Additionally, modders might not increment the version number. It would be better to compare the xxhash of the files
- [ ] The downloading tasks are not running concurrently, even though the process is optimized
- [x] The list command displays the basename of the filename, instead of the actual mod name
- [ ] The `LastUpdate` value is not in a human-readable format

## TODO

- [x] Implement logger
- [x] Implement custom errors
- [x] Implement `fetch_mod_registry()` in `Downloader` struct instead of using `ModCatalog::fetch_from_network()`
- [ ] Implement `has_matching_hash()` instead of version comparison
- [ ] Fix concurrent downloading by using `tokio::spawn`
- [ ] Update example output at usage section in `README.md`

## Motivation

Everest and Olympus are excellent tools for managing Celeste mods. However, there are still some quality-of-life improvements that could be made:

- In some countries, the download speed in the game menu is quite slow, and there's no option to accelerate it (e.g., using `aria2c`).
- In the mod menu, there is no way to **cancel** or **pause/resume** downloads.
- When uninstalling specific mods, it isn’t clear which *dependencies* are still needed by other mods.

> I noticed that when using a web browser or CLI tools like `curl` or `wget`, the download speed is much faster than in-game. This suggests that there might be issues in the original code or libraries.

## Features

- Easy mod installation and management
- Search for mods in the Everest online database by name
- Display detailed information about a specific mod
- Install a mod by its name
- List all installed mods
- Show the details of a specific mod that have been installed
- Check for available updates for installed mods

## Installation

Make sure you have installed [Rust](https://www.rust-lang.org/tools/install) and [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html).

Clone the repo and build it yourself using `cargo`.

```sh
git clone https://github.com/pinpinroku/everest-mod-cli.git
cd everest-mod-cli
cargo build --release
```

> If your CPU supports the **AVX2** feature, set the flag `RUSTFLAGS='-C target-feature=+avx2'` before building to accelerate hash calculation speed. You can check whether your CPU supports **AVX2** by running `lscpu | grep avx2`.

Then symlink the binary to the local bin directory.

```sh
mkdir -p ~/.local/bin/
ln -s $HOME/everest-mod-cli/target/release/everest-mod-cli $HOME/.local/bin/everest-mod-cli
```

## Usage

```bash
everest-mod-cli [COMMAND] [OPTIONS]
```

Available commands:

### `search <query>`
Search for mods in the Everest online database using a search query.
```bash
everest-mod-cli search "GDDH"
# Fetching remote mod registry...
# Found 1 matching mods:
#
# GDDH (v1.0.3)
#  Last updated: 1718743274
#  Download link: https://gamebanana.com/mmdl/1218505
```

### `info <mod_name>`
Display detailed information about a specific mod.
```bash
everest-mod-cli info "SpringCollab2020"
# Fetching remote mod registry...
# SpringCollab2020 (v1.7.9)
# Last updated: 1725474405
# Download link: https://gamebanana.com/mmdl/1273559
# Page URL: https://gamebanana.com/mods/584862
# Hashes: e944c53a9a64f7e8
```

### `install <mod_name>`
Install a mod by its name. The mod will be downloaded and installed in the appropriate directory.
Checksum verification is performed automatically to ensure the integrity of the downloaded mod.
```bash
everest-mod-cli install "SpringCollab2020"
```

### `list`
List all installed mods, showing their versions and dependencies.
```bash
everest-mod-cli list
# Installed mods:
#   Iceline_silentriver v1.1
#   aqualias-101 v1.0.1
```
> This command will show you the filename of the mod, not the mod name.

### `show <mod_name>`
Show the details of a specific mod that have been installed.
```bash
everest-mod-cli show "Iceline_silentriver"
# Name: Iceline_silentriver
# Version: 1.1

# Dependencies:
#   - Everest v1.4.0.0
#   - SkinModHelper v0.6.1
#   - IcelineLoadingAnim v1.0.0
```

### `update`
Check for available updates for installed mods.
```bash
# Check for updates
everest-mod-cli update
# Check and install available updates
everest-mod-cli update --install
```

## Notes

- The `mod_name` and their filenames are not the same.
- The `mod_name` is the name of the Mod as it appears in the game menu.
- The `filename` is the name of the file that contains the Mod's metadata.

## Credits

This project uses [Everest](https://github.com/EverestAPI/Everest), the Celeste Mod Loader and Base API.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
