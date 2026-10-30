//! Express-compatible web plumbing shared by the admin dashboards: the
//! request/response behaviour of Express and body-parser (`http`),
//! express-session's MemoryStore (`session`), csurf's cookie mode (`csrf`),
//! express-rate-limit's memory store (`limiter`) and EJS-compatible page
//! rendering with minijinja (`views`).

pub mod csrf;
pub mod http;
pub mod limiter;
pub mod session;
pub mod views;
