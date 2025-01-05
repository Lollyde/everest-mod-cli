# Everest Mod CLI

A command-line interface tool for managing Celeste mods using Everest API. WIP.

## Features

- Easy mod installation and management
- Command-line interface for automation

## Installation

[Installation instructions will be added]

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
