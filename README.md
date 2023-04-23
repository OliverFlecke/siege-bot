# Discord bot for Rainbow Six Siege statistics

[![Test](https://github.com/oliverflecke/siege-bot/actions/workflows/test.yml/badge.svg)](https://github.com/oliverflecke/siege-bot/actions/workflows/test.yml?query=branch%3Amain)
[![Images](https://github.com/oliverflecke/siege-bot/actions/workflows/publish_image.yml/badge.svg)](https://github.com/oliverflecke/siege-bot/actions/workflows/publish_image.yml?query=branch%3Amain)
[![Codecov](https://codecov.io/github/oliverflecke/siege-bot/coverage.svg?branch=main)](https://codecov.io/gh/oliverflecke/siege-bot)
[![Dependency status](https://deps.rs/repo/github/oliverflecke/siege-bot/status.svg)](https://deps.rs/repo/github/oliverflecke/siege-bot)

This repository contains a crate for connecting to Ubisoft's API to retreive statistics about players in Rainbow Six Siege, along with a Discord bot for interacting it with it.

## Build and running

To run the Discord bot, the following environment variables have to be set.

```sh
export UBISOFT_EMAIL="<email for ubisoft account>"
export UBISOFT_PASSWORD="<password for ubisoft account>"
export DISCORD_TOKEN="<token for discord bot>"
```

The bot can then be run with from the root of the repository with `cargo run siege-bot`.

In order to link Discord IDs to Ubisoft accounts between restarts, the bot will store these relationships in a json file. It will first look relative to itself for `.players.json` or secondly at `/config/.players.json`. The second one was added to support mounting the file inside a container.

### Logs

Logs will by default be outputted to stdout and written to daily rolling files in `./logs/`. This directory can be customized by setting `LOGS_DIR` in the environment. If running inside a container, this must be mounted to a host machines directory to be persisted.

## Running inside container

To run the bot as a container, the following comand can be used:

```sh
docker run --env-file .env -v ${PWD}/config:/config --name siege-bot -d ghcr.io/oliverflecke/siege-bot
```

Note that the format for the `.env` file should just be a `key=value` pair on each line (no `export` or qoutes).

The `config` mount is optional, but necessary to persist players' links between their Ubisoft accounts and Discord ID.

## Development

All relevant checks for the code will be done and validated through Github workflows, but to avoid extra runs a [pre-commit](https://pre-commit.com) config is included in the repository. Install the tool and run `pre-commit install` to setup git hooks to run before each commit. The checks can also be run manually with `pre-commit run`.

### Tests

To run all the tests, the environment variables for Ubisoft have to set, as it is required to testing the interaction with the API.
