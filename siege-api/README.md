# Crate for Rainbow 6 Siege

Contains a Rust crate for interacting with Ubisoft's API for Rainbow 6 Siege.

## Development

Tested with Rust v1.68.0.

## Authentication

Authentication is required to communicate with the API. This requires an email and password for an Ubisoft account. These should be exposed as environment variables, eg:

```sh
export UBISOFT_EMAIL="<your email>"
export UBISOFT_PASSWORD="<your password>"
```
