# tento-server

Rust backend API for Tento (Actix-web + async-graphql + MongoDB).

## Common Commands

```bash
cd components/api/tento-server

# Build
cargo build

# Run
cargo run

# Tests
cargo test
cargo test --test integration_tests
cargo test --test repository_contract_tests

# Lint / format
cargo clippy
cargo fmt -- --check
```

## Environment Variables

Required server environment variables:

- MONGO_CONN_STRING
- MONGO_DB_NAME
- USERS_COLLECTION
- GH_CLIENT_ID
- GH_CLIENT_SECRET
- WEB_SERVER_HOST
- WEB_SERVER_PORT
- JWT_SECRET
- JWT_EXPIRATION_HOURS
- FUNC_ENUMS_EMBED_MODEL
- FUNC_ENUMS_MAX_RESPONSE_TOKENS
- FUNC_ENUMS_MAX_REQUEST_TOKENS
- FUNC_ENUMS_MAX_FUNC_TOKENS


