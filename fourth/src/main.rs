#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use eframe::{self, egui};

use self::flow_control::AZCharFourth;

fn main() {
    let mut options = eframe::NativeOptions::default();
    eframe::run_native(
        "AZChar-Fusion",
        options,
        Box::new(|_cc| Box::new(AZCharFourth::default())),
    );
}

mod backend;
mod flow_control;
mod frontend;
