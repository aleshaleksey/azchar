
// #![cfg_attr(
//   all(not(debug_assertions), target_os = "windows"),
//   windows_subsystem = "windows"
// )]
use azchar_server::{Request, Response};
#[macro_use]
extern crate tauri;
use tauri::{Builder};


fn main() {
  tauri::Builder::default()
    .run(tauri::generate_context!(
      "fourth/src/tauri.conf.json"
    ))
    .expect("error while running tauri application");
}

mod frontend;
mod backend;
