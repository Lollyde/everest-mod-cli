# Everest Mod CLI

**WIP**.

A command-line interface tool for managing Celeste mods using Everest API.

Currently, target **Steam** and **Linux** installation.

## Motivation

Everest and Olympus are great tool to managing Celeste mods.

However, some QoLs are missing:

- The download speed in the game menu is too slow in some countries and no way to accelarate them like using `aria2c`.
- In the mod menu of the game we can't *cancel* or *pause/resume* download process.
- We couldn't find out which *dependencies* are still needed by another mods when uninstall specific mods.

> I found the download speed is not so slow like in-game when I using web browser or CLI tools like `curl` or `wget`.

## Features

- Easy mod installation and management
- Command-line interface for automation

## Installation

Clone this repo and build it yourself using rust.

```sh
git clone https://github.com/pinpinroku/everest-mod-cli.git
cd everest-mod-cli
cargo build --release
```

Then symlink the binary to the local bin directory.

```sh
mkdir -p ~/.local/bin/
cd ~/.local/bin/
ln -s /home/{username}/everest-mod-cli/target/release/everest-mod-cli modmgr
modmgr --help
```

> Replace `{username}` to actual username. Chose your prefered filename for symlink.

## Usage

```bash
everest-mod-cli [COMMAND] [OPTIONS]
```

Available commands:

### `search <query>`
Search for mods in the Everest catalog using a search query.
```bash
everest-mod-cli search "map"
```

### `info <name>`
Display detailed information about a specific mod.
```bash
everest-mod-cli info "SpringCollab2020"
```

### `install <name>`
Install a mod and its dependencies.
```bash
everest-mod-cli install "SpringCollab2020"
# Skip checksum verification
everest-mod-cli install "SpringCollab2020" --no-verify
```

### `list`
List all installed mods, showing their versions and dependencies.
```bash
everest-mod-cli list
```

### `update`
Check for available updates for installed mods.
```bash
# Check for updates
everest-mod-cli update
# Check and install available updates
everest-mod-cli update --install
```

### Global Options
- `--test`: Use test YAML file instead of fetching from network

## Credits

This project uses [Everest](https://github.com/EverestAPI/Everest), the Celeste Mod Loader and Base API.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
