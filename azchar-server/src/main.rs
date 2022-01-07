//! This module deals with server related things. Namely with the interactions
//! with the outside world.
//! This includes:
//! a) The main loop.
//! b) Requests and responses.
#![allow(clippy::field_reassign_with_default)]
extern crate serde_json;
#[cfg(test)]
extern crate tempfile;
extern crate toml;
#[macro_use]
extern crate serde_derive;

extern crate azchar_config;
extern crate azchar_database;
extern crate azchar_error;

mod main_loop;
mod requests;

use crate::main_loop::MainLoop;
use crate::main_loop::Mode;

// macro_rules! do_or_die {
//     ($result:expr) => {
//         match $result {
//             Ok(r) => r,
//             Err(e) => {
//                 println!("Big fail: {:?}", e);
//                 return;
//             }
//         }
//     };
// }

fn main() {
    // Get settings.
    let args: Vec<String> = std::env::args().map(String::from).collect();
    let address: String = match args.get(1) {
        Some(s) => String::from(s),
        None => String::from("127.0.0.1:55555"),
    };
    let mode = args
        .iter()
        .map(|x| Mode::from_args(x))
        .find(|x| !matches!(x, Mode::Default))
        .unwrap_or(Mode::Default);

    match MainLoop::create_with_connection(&address) {
        Ok(mut ml) => ml.run(mode),
        Err(e) => println!("{}", e),
    }
}
