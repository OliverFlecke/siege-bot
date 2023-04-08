# Discord bot for Rainbow 6 Siege statistics

This repository contains a crate for connecting to Ubisoft's API to retreive statistics about players in Rainbow 6 Siege, along with a Discord bot for interacting it with it.

Implemented with Rust v1.68 (or properly later).

## Build and running

To run the Discord bot, the following environment variables have to be set.

```sh
export UBISOFT_EMAIL="<email for ubisoft account>"
export UBISOFT_PASSWORD="<password for ubisoft account>"
export DISCORD_TOKEN="<token for discord bot>"
```

The bot can then be run with from the root of the repository with `cargo run siege-bot`.

In order to link Discord IDs to Ubisoft accounts between restarts, the bot will store these relationships in a json file. It will first look relative to itself for `.players.json` or secondly at `/config/.players.json`. The second one was added to support mounting the file inside a container.

## Running inside container

To run the bot as a container, the following comand can be used:

```sh
docker run --env-file .env -v ${PWD}/config:/config --name siege-bot -d ghcr.io/oliverflecke/siege-bot
```

Note that the format for the `.env` file should just be a `key=value` pair on each line (no `export` or qoutes).

The `config` mount is optional, but necessary to persist players' links between their Ubisoft accounts and Discord ID.

## Tests

To run all the tests, the environment variables for Ubisoft have to set, as it is required to testing the interaction with the API.
