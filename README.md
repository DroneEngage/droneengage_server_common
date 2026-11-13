# droneengage_server_common

Shared code of the DroneEngage Rust servers (authenticator, comm server,
storage server). It replaces the Node.js `droneengage_server_common` npm
package, whose behaviour the modules keep:

| Module | From the npm package | What it does |
|---|---|---|
| `config` | `js_serverConfig.js`, `hlp_args.js`, `js_3rd_StripJsonComments.js` | `server.config` loading: JSON with comments, `<file>.local` overlay, environment overrides, `$$HASH$$` handling; command line arguments |
| `config_handler` | `js_config_handler.js` | `$$HASH$$('plaintext')` directives become bcrypt hashes, in memory and in the file |
| `validation` | `hlp_validation.js` | alphanumeric, email, version and admin credential checks |
| `password` | `hlp_password.js` | bcrypt access codes with legacy plaintext upgrade |
| `strings` | `hlp_strings.js`, `js_styleHelper.js` | random strings, padding, traffic CSS class |
| `colors` | `js_colors.js` | ANSI console colors |
| `util`, `js` | (the servers' JavaScript semantics) | JSON values handled like JavaScript did: `String()`, `Number()`, `parseInt`, `JSON.stringify` output, property access errors |
| `logger` | (`node-file-logger`) | the daily `Logs_YYYY-MM-DD.log` file logger |
| `net` | (`https`, `ws`) | TLS acceptors and connectors, WebSocket limits |
| `s2s_auth` | (the servers' `js_s2s_auth.js`) | Ed25519 / HMAC-SHA256 server-to-server authentication |

## Use

In a server's `Cargo.toml`:

```toml
[dependencies]
droneengage_server_common = { git = "https://github.com/DroneEngage/droneengage_server_common", branch = "master" }
```

Loading a configuration with the server's own environment overrides:

```rust
use droneengage_server_common::config::{Config, EnvOverride, LoadOptions};

fn debug(cfg: &mut serde_json::Map<String, serde_json::Value>, v: &str) {
    cfg.insert("debug_logging".into(), (v == "true" || v == "1").into());
}

let overrides = [
    ("DE_MY_ENV", EnvOverride::Key("config_key")),
    ("DE_MY_FLAG", EnvOverride::Transform(debug)),
];
let cfg = Config::load_with(
    &std::env::current_dir()?,
    "server.config",
    &LoadOptions { env: &|k| std::env::var(k).ok(), log: &|l| println!("{}", l), overrides: &overrides, hash_handling: true },
)?;
```

## Test

```bash
cargo test
```

The Node.js package stays available on `master` and its tags (`v1.0.1`) for
the JavaScript servers.
