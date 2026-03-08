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

- ALLOWED_REDIRECT_ORIGINS (comma-separated origins, e.g. `https://app.example.com,https://localhost:5173`) - allowed origins used to validate OAuth redirect_uri values.

### Redirect URI handling and security

- The server validates any `redirect_uri` passed by callers during the OAuth flow against the canonicalized origins in `ALLOWED_REDIRECT_ORIGINS`.
- Origins are canonicalized at startup (scheme://host[:port]) via the `Config::canonicalize_origin` helper to ensure consistent comparisons.
- If a `redirect_uri` is provided and its origin matches an entry in the allowlist, the exact provided `redirect_uri` is used in the OAuth token exchange (GitHub requires an exact redirect_uri match).
- If `redirect_uri` is omitted, the server constructs a fallback using the first allowlisted origin with the path `/auth/callback`.
- Do not include secrets in logs; the code avoids logging secret contents or lengths.

### REST vs GraphQL authorization policy

- The application uses `claims.sub` (the JWT subject — either ObjectId hex string or username) as the canonical user identifier for ownership checks.
- Authorization helpers live in `src/auth/utils.rs`:
  - `require_owner_or_admin(claims, resource_owner)` — allows owners and admins.
  - `require_quiz_owner(claims, quiz_owner_id)` — enforces quiz ownership (admins bypass).
  - `require_user_owner(claims, username, user_repository)` — resolves username → id and enforces owner/admin semantics.
- REST handlers use `AuthenticatedUser` (extracted from request extensions by middleware) and compare `claims.sub` to resource owner ids; GraphQL resolvers use `extract_claims_from_context(ctx)` and the same helper functions. This keeps enforcement consistent across both APIs.

Additions in this release:
- Redirect origin canonicalization and robust allowlist validation.
- Unit tests covering redirect allowlist behavior and ownership helper tests.

