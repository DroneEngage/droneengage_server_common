# AGENTS.md — droneengage_server_common

Shared Rust crate of the DroneEngage servers (authenticator, comm
server, storage server, udp proxy) — the replacement for the old
`droneengage_server_common` npm package, whose behaviour it keeps. A
library, not a binary: the servers depend on it via git (branch
`pr_rust`) or a path dep. See `../../AGENTS.md`.

    cargo build
    cargo test

## What it provides

- `config` — `server.config` loading: JSON with C-style comments,
  `<file>.local` overlay merge, environment overrides, `$$HASH$$`
  handling, command-line arguments.
- `config_handler` — `$$HASH$$('plaintext')` → bcrypt directives.
- `validation` / `password` — credential checks; bcrypt access codes
  with legacy plaintext upgrade.
- `strings`, `colors`, `util`, `js` — random strings, padding, ANSI
  colors, JavaScript-compatible JSON value coercion (`String()`,
  `Number()`, `parseInt`, `JSON.stringify` output).
- `logger` — the daily `Logs_YYYY-MM-DD.log` file logger.
- `net`, `ws_accept`, `web` — rustls TLS acceptors/connectors,
  WebSocket limits, HTTP/template helpers.
- `s2s_auth` — Ed25519 / HMAC-SHA256 server-to-server authentication.

`src/` mirrors that list one file per module. `README.md` has the full
npm→Rust mapping table and a usage example.
