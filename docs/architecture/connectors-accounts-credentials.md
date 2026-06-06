# Connectors, Accounts, And Credentials

OpenNivara needs a connector/account/credential foundation before authenticated external mutation tools.

Unauthenticated external read tools, such as `http_get`, may land earlier. Authenticated tools and external side-effect tools must wait for this foundation.

## Concepts

Connector:

- integration type
- examples: `github`, `gmail`, `slack`, `discord`, `generic_http`, `telegram_bot`
- describes service identity, auth methods, capabilities, and tool exposure

Account:

- connected identity for a connector
- examples: GitHub account, mail account, workspace/user account
- one connector can have multiple accounts

Credential:

- token, API key, or secret material for an account
- SQLite stores metadata only
- OS keychain stores secret/token material

## Storage Rule

State DB stores connector, account, and credential metadata. OS keychain stores secret material.

Do not store raw OAuth access tokens, refresh tokens, GitHub tokens, Slack tokens, mail tokens, bot tokens, or API keys directly in SQLite.

`secrets.toml` may remain for Gemini/dev-alpha. Long-term connector credentials should use the `CredentialStore` abstraction and OS keychain.

Configured model-provider calls remain automatic. Sending prompt/context to the configured model provider is not treated like arbitrary external mutation.

## Target Modules

```text
src/connectors/
  mod.rs
  types.rs
  registry.rs
  credentials.rs
  oauth.rs
```

`types.rs`:

- connector/account/credential DTOs
- account and credential statuses
- scope metadata

`registry.rs`:

- connector definitions
- connector capability definitions
- enabled connector lookup

`credentials.rs`:

- `CredentialStore` trait
- keychain/dev/env/mock implementations
- redaction helpers

`oauth.rs`:

- OAuth2 Authorization Code + PKCE flow
- device-code flow where supported
- callback/deep-link state validation

## CredentialStore

```rust
pub trait CredentialStore {
    fn put_secret(
        &self,
        key: &CredentialKey,
        secret: SecretString,
    ) -> Result<(), CredentialError>;

    fn get_secret(
        &self,
        key: &CredentialKey,
    ) -> Result<SecretString, CredentialError>;

    fn delete_secret(
        &self,
        key: &CredentialKey,
    ) -> Result<(), CredentialError>;
}
```

Implementations:

- `OsKeychainCredentialStore`
- `EnvCredentialStore`
- `DevFileCredentialStore`
- `MockCredentialStore`

`MockCredentialStore` is required before real connector tests. `DevFileCredentialStore` must be opt-in and clearly marked as development-only.

## Dependencies

Add these when implementation starts:

```toml
keyring = "4"
secrecy = { version = "0.10", features = ["serde"] }
zeroize = { version = "1.8", features = ["derive"] }
oauth2 = { version = "5", default-features = false, features = ["reqwest"] }
```

Use:

- `keyring` for OS keychain integration
- `secrecy` for safer in-memory secret wrappers
- `zeroize` for wiping secret memory where possible
- `oauth2` for OAuth2 Authorization Code + PKCE/device flow
- existing `reqwest` for HTTP

Do not add yet:

- `axum`
- `warp`
- `openidconnect`
- GraphQL client frameworks
- GitHub/Gmail/Slack SDKs
- generic plugin framework

Exception: add an SDK when implementing a real connector that needs it.

SDK rule: use SDKs when they are maintained, typed, and help with auth, pagination, rate limits, uploads, or API complexity. Use handwritten `reqwest` when an endpoint is tiny/stable or when the SDK hides details needed for approval previews.

Likely future choices:

- GitHub: consider `octocrab`
- Slack: consider `slack-morphism`
- Gmail: evaluate a Gmail crate versus handwritten `reqwest`

Do not handwrite large APIs by default. Do not add large SDKs before the connector abstraction exists.

## Web Framework Rule

No web framework now.

If an OAuth callback server becomes necessary later:

1. first try Desktop deep link, loopback flow, or CLI device-code flow
2. prefer `axum` over `warp` if a local callback server is still needed

Telegram should not run OAuth flows directly. Telegram should say:

```text
Connect this account from Desktop or CLI first.
```

Then Telegram may use already-connected accounts if same local owner/session policy allows it.

## Connector DB Target

Later migration: `connector_accounts`

```text
- id
- connector_id
- display_name
- external_account_id
- email
- status
- scopes_json
- created_at
- updated_at
- last_verified_at
- metadata_json
```

Later migration: `connector_credentials`

```text
- id
- account_id
- kind
- keychain_service
- keychain_key
- scopes_json
- expires_at
- status
- created_at
- updated_at
```

Credential kinds:

- `oauth2_token`
- `api_key`
- `bot_token`

Account statuses:

- `connected`
- `disabled`
- `expired`
- `revoked`
- `error`

Credential statuses:

- `active`
- `expired`
- `revoked`
- `missing`

## Redaction

Credential metadata can be logged only after redaction. Secret material must never appear in:

- logs
- approval previews
- model-visible tool results
- prompt audits
- pending turn JSON
- connector metadata JSON

Approval previews may show connector ID, capability ID, account display name, required scopes, and target. They must not show token values or API keys.

## Implementation Milestones

PR 1 establishes types and storage contracts:

- add `src/connectors/types.rs`
- add connector/account/credential status types
- add `CredentialKey`
- add DB metadata schema plan
- add serialization tests

PR 2 adds credential store abstraction:

- add `CredentialStore`
- add `MockCredentialStore`
- add `EnvCredentialStore`
- add redaction tests

PR 3 adds OS keychain support:

- add `keyring`, `secrecy`, and `zeroize`
- implement `OsKeychainCredentialStore`
- keep dev file storage opt-in

PR 4 adds OAuth helpers:

- add `oauth2`
- implement Authorization Code + PKCE state helpers
- implement device-code helper where providers support it
- keep Telegram OAuth disabled

PR 5 wires connector metadata into state DB:

- add `connector_accounts`
- add `connector_credentials`
- add account status transitions
- add credential status transitions

## Locked Decisions

1. Design connector/account/credential store before external mutation tools.
2. Connector means integration type.
3. Account means connected identity for a connector.
4. Credential means secret/token metadata in DB plus secret material in OS keychain.
5. Do not store raw tokens/API keys directly in SQLite.
6. Use `keyring` for OS keychain when implemented.
7. Use `oauth2` for OAuth2 Authorization Code + PKCE/device flow when implemented.
8. Use `secrecy` and `zeroize` for in-memory secret handling.
9. Do not add a web framework yet.
10. Use SDKs for real connectors when they are good, maintained, and useful.
11. Do not add GitHub/Gmail/Slack SDKs before connector abstraction exists.
12. Telegram can use connected accounts only after Desktop/CLI connection flow.
13. Generic authenticated `api_request` waits until connector/auth/redaction is mature.

## Tests

Required tests:

1. Credential metadata can be stored without secret material.
2. `MockCredentialStore` can put/get/delete secrets.
3. Account can transition between connected, disabled, expired, revoked, and error states.
4. Credential can transition between active, expired, revoked, and missing states.
5. Connector account scopes round-trip through JSON.
6. Redaction prevents credentials from appearing in logs, previews, and model-visible results.
7. Telegram cannot start OAuth directly and returns a connect-from-Desktop/CLI message.
8. SDK-specific code is not required until a connector implementation lands.
