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

## Tests

To run all the tests, the environment variables for Ubisoft have to set, as it is required to testing the interaction with the API.

