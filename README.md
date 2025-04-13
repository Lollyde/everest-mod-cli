# Everest Mod CLI

**WIP**. This project is currently under development. Use at your own risk.

A command-line interface tool for managing Celeste mods using Everest API.

Currently, target **Steam** and **Linux** installation.

## Motivation

Everest and Olympus are great tool to managing Celeste mods.

However, some QoLs are missing:

- The download speed in the game menu is too slow in some countries and no way to accelerate them like using `aria2c`.
- In the mod menu of the game we can't **cancel** or **pause/resume** the download process.
- We couldn't find out which *dependencies* are still needed by another mods when uninstall specific mods.

> I found the download speed is not so slow like in-game when I using web browser or CLI tools like `curl` or `wget`. I think there might be some issues in the original `C#` codes or libraries.

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
# Fetching mod list from: https://maddie480.ovh/celeste/everest_update.yaml
# Found 1 matching mods:
#
# GDDH (v1.0.3)
#  Last updated: 1718743274
#  URL: https://gamebanana.com/mmdl/1218505
```

### `info <mod_name>`
Display detailed information about a specific mod.
```bash
everest-mod-cli info "SpringCollab2020"
# Fetching mod list from: https://maddie480.ovh/celeste/everest_update.yaml
# SpringCollab2020 (v1.7.9)
# Last updated: 1725474405
# URL: https://gamebanana.com/mmdl/1273559
# GameBanana ID: Some(150813)
# Hash: e944c53a9a64f7e8
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

- The mod name and their filenames are not the same.
- The mod name is the name of the mod as it appears in the game menu.
- The filename is the name of the file that contains the mod's metadata.

## Credits

This project uses [Everest](https://github.com/EverestAPI/Everest), the Celeste Mod Loader and Base API.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
