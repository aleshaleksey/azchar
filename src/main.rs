#![allow(clippy::field_reassign_with_default)]
extern crate serde_json;
#[cfg(test)]
extern crate tempfile;
extern crate toml;
extern crate uuid_rs;
#[macro_use]
extern crate serde_derive;
extern crate fnv;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate diesel;

mod config;
mod database;
mod error;
mod roller;
mod server;

use crate::database::LoadedDbs;
use crate::server::main_loop::Mode;
use crate::server::MainLoop;

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
