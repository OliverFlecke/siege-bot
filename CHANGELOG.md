# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.3]

### Changed

- Added `NoClass` operator roll, which the API has started to return.

## [0.7.2]

### Changed

- Disable rolling of log, as it is not really required for the amount of requests at the moment.

## [0.7.1]

### Added

- Support for reworked map `CONSULATE V2`.

## [0.7.0]

### Added

- Support for season Y8S2 and operator Fenrir.

## [0.6.0]

### Changed

- `all_maps` and `all_operators` now includes a `game_mode` option for either `all` or `ranked` to filter data.

## [0.5.1]

### Changed

- Statistics include rank information

## [0.5.0]

### Changed

- Statistics client API now returns top level data, with getters to retreive relevant data.
- Added choices for gamemode and platform for Discord `statistics` command

## [0.4.0]

### Added

- Writing logs to file, customizable with environment var `LOGS_DIR`

## [0.3.0]

### Added

- Command to get the status of Siege's servers with `/status`

## [0.2.0]

### Added

- Command to sending player statistics to Discord
- Command to send maps statistics to Discord with `/map <map_name>`
- Command to send operator statistics to Discord with `/operator <name>`
- Linking Siege players with their Discord account with `/add`
- Command to send statistics about all maps for a given player, with multiple sorting options
- Command to send statistics about all operators for a given player, with a side and multiple sorting options
- Autocomplete for entering map and operator names

## [0.1.0]

### Added

- Small framework for Discord bot.
- Simple ping and ID command to verify command is alive.
- API: Retrieving statistics from players through the ranked v2 endpoint.
