#![allow(clippy::field_reassign_with_default)]
extern crate serde_json;
extern crate toml;
#[macro_use]
extern crate serde_derive;
extern crate websocket;

extern crate azchar_config;
extern crate azchar_database;
extern crate azchar_error;
pub mod requests;
pub use requests::{Request, Response};
