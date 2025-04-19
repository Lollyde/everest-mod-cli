# Everest Mod CLI

**WIP**: This project is under development. Expect breaking changes and limited functionality. Use at your own risk.

A command-line interface tool for managing Celeste mods using the maddie's public online database.

Currently, target **Linux** installation. Flatpak version is not supported.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
  - [search](#search-query)
  - [info](#info-mod_name)
  - [install](#install-mod_name)
  - [list](#list)
  - [show](#show)
  - [update](#update)
- [Motivation](#motivation)
- [Notes](#notes)
- [Credits](#credits)
- [License](#license)

## Issues

- [x] `ModCatalog::fetch_from_network()` is being called twice in some operations
- [x] Version comparison is not meaningful. The value might contain a nonsensical string instead of a number. Additionally, modders might not increment the version number. It would be better to compare the xxhash of the files
- [ ] The downloading tasks are not running concurrently, even though the process is optimized
- [x] The list command displays the basename of the filename, instead of the actual mod name
- [ ] The `LastUpdate` value is not in a human-readable format
- [ ] Dependencies does not shown in `show` command
- [ ] Does not respect `updaterblacklist.txt`
- [ ] No caches for remote mod registry and manifests files

## TODO

- [x] Implement logger
- [x] Implement custom errors
- [x] Implement `fetch_mod_registry()` in `Downloader` struct instead of using `ModCatalog::fetch_from_network()`
- [x] Implement `has_matching_hash()` instead of version comparison
- [ ] Fix concurrent downloading by using `tokio::spawn`
- [ ] Update example output at usage section in `README.md`
- [ ] Verify checksum of downloaded file on temporary directory to avoid needless disk allocation
- [ ] Move Issues and TODO to GitHub issue page
- [ ] Update credits section, reference to maddie's repo
- [ ] Think about more suitable name for the program

## Features

- Easy mod installation and management within terminal
- Install a mod by its name: No need for Olympus or browser
- List all installed mods with actual mod name and version
- Search for mods by name: Quickly find mods in the Everest online database without navigating through a browser.
- Display detailed information about a specific mod: Useful for manual downloads using external downloader like `wget` or `aria2c`
- Show the details of a specific mod that have been installed: Easy to check their dependencies
- Check for available updates for installed mods and can install them automatically: Running on background! You can play the game while downloading!
- Asynchronous downloads reduce total download time, also lesser memory usage and faster checksum veryfications

## Installation

Make sure you have installed [Rust](https://www.rust-lang.org/tools/install) and [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html).

Clone the repo and build it yourself using `cargo`.

```bash
git clone https://github.com/pinpinroku/everest-mod-cli.git
cd everest-mod-cli
cargo build --release
```

> If your CPU supports the **AVX2** feature, set the flag `RUSTFLAGS='-C target-feature=+avx2'` before building to accelerate hash calculation speed. You can check whether your CPU supports **AVX2** by running `lscpu | grep avx2`.

Then symlink the binary to the local bin directory.

```bash
mkdir -p ~/.local/bin/
ln -s $HOME/everest-mod-cli/target/release/everest-mod-cli $HOME/.local/bin/everest-mod-cli
```

> We plan to release the built binary files once the specifications are finalized (stabilized).

## Usage

```bash
everest-mod-cli [OPTIONS] [COMMAND] 
```

Available commands:

### `search <query>`

Search for mods in the online database using a search query.
```bash
everest-mod-cli search "GDDH"
# Searching for mods matching 'shrimp'...
# Found 8 matching mods:
# 
# ShrimpGlider (version 1.0.0)
#  - Updated at: 1680913152
#  - Page: https://gamebanana.com/mods/436804
#  - Download: https://gamebanana.com/mmdl/962758
# 
# Shrimptember2nd (version 1.0.0)
#  - Updated at: 1730077291
#  - Page: https://gamebanana.com/mods/521722
#  - Download: https://gamebanana.com/mmdl/1309084
# 
# ShrimpHelper (version 1.2.3)
#  - Updated at: 1743778527
#  - Page: https://gamebanana.com/mods/435408
#  - Download: https://gamebanana.com/mmdl/1414732
#
# ------------------------------------------------
#
```

### `info <mod_name>`

Display detailed information about a specific mod.
```bash
everest-mod-cli info "SpringCollab2020"
# Looking up information for the mod 'zbs_Crystal'...
# 
# zbs_Crystal (version 1.2.8)
#  - Updated at: 1735987004
#  - Page: https://gamebanana.com/mods/468140
#  - Download: https://gamebanana.com/mmdl/1356216
#  - Hashes: c122676ef89c310d
```

### `install <mod_name>`

Install a mod by its name. The mod will be downloaded and installed in the appropriate directory.
Checksum verification is performed automatically to ensure the integrity of the downloaded mod.
```bash
everest-mod-cli install "SpeedrunTool"
# Starting installation of the mod 'SpeedrunTool'...
# Downloading mod files...
#   [00:00:08] [################################################] 245.41 KiB/245.41 KiB (0s)
# Verifying checksum...
# Checksum verified
# Installation finished successfully!
```

### `list`

List all installed mods, showing their actual names and versions.
```bash
everest-mod-cli list
# Collecting information about installed mods... This might take a few minutes if your mods library is huge
#
# Installed mods:
#  - Iceline_silentriver v1.1
#  - aqualias-101 v1.0.1
```

### `show <mod_name>`

Show the details of a specific mod that have been installed.
```bash
everest-mod-cli show "Iceline_silentriver"
# Checking installed mod information...
# Mod Information:
# - Name: Iceline_silentriver
# - Version: 1.1
#
# Dependencies:
#  - Everest v1.4.0.0
#  - SkinModHelper v0.6.1
#  - IcelineLoadingAnim v1.0.0
```

### `update`

Check for available updates for installed mods.
```bash
# Check for updates
everest-mod-cli update
# Checking mod updates...
# Available updates:
# 
# StrawberryJam2021
#  - Current version: 1.0.11
#  - Available version: 1.0.12
# 
# Run with --install to install these updates
```
```bash
# Check and install available updates
everest-mod-cli update --install
# Checking mod updates...
# Available updates:
# 
# StrawberryJam2021
#  - Current version: 1.0.11
#  - Available version: 1.0.12
# 
# Installing updates...
# 
# Updating StrawberryJam2021...
#   [00:03:26] [################################################] 91.22 MiB/91.22 MiB (0s)
#   Verifying checksum...
#   Checksum verified!
#   Updated StrawberryJam2021 to version 1.0.12
# 
# All updates installed successfully!
```

## Option

You can specify your mods directory using `--mods-dir`.
```bash
# Install the mod "SpeedrunTool" while specifying the mods directory
everest-mod-cli --mods-dir /home/maddy/game/exokgames/celeste/Mods/ install "SpeedrunTool"
```
> The directory should have permissions of at least 0700.

## Motivation

Everest and Olympus are excellent tools for managing Celeste mods. However, there are still some quality-of-life improvements that could be made:

- Olympus is unstable or completely non-functional on certain Linux distributions, particularly in **Wayland** environments.
- Download speed is slow in some countries.
  - CLI tools like curl or wget are sometimes faster than in-game downloads.
  - Cannot run auto updates on background
  - Need to wait or entirely cancel the updates when opening the game
- No option to *cancel*, *pause*, or *resume* downloads in mod menu.
- Lack of clarity about `dependencies` when uninstalling mods.

## Notes

- The `mod_name` and their filenames are not the same.
- The `mod_name` is the name of the Mod as it appears in the game menu.
- The `filename` is the name of the file that contains the Mod's metadata.

## Credits

This project uses [Everest](https://github.com/EverestAPI/Everest), the Celeste Mod Loader and Base API.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
