//! droneengage_server_common: the code the DroneEngage Rust servers share
//! (authenticator, comm server, storage server).
//!
//! - `config`, `config_handler`: server.config loading (comments, `.local`
//!   overlay, environment overrides, `$$HASH$$` directives)
//! - `util`, `js`: JSON handled the way the original JavaScript servers
//!   handled it (coercions, `JSON.stringify` output, property access)
//! - `logger`: the daily file logger (node-file-logger's format)
//! - `net`: TLS acceptors and connectors, WebSocket settings
//! - `s2s_auth`: Ed25519 / HMAC server-to-server authentication
//! - `validation`, `password`, `strings`, `colors`: helpers

pub mod colors;
pub mod config;
pub mod config_handler;
pub mod js;
pub mod logger;
pub mod net;
pub mod password;
pub mod s2s_auth;
pub mod strings;
pub mod util;
pub mod validation;
