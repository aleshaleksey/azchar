#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use eframe;
use egui::containers::Frame;

use self::flow_control::AZCharFourth;

fn main() {
    let mut options = eframe::NativeOptions::default();
    options.initial_window_size = Some(egui::Vec2::new(720., 768.));

    let f = Frame::none()
        .fill(egui::Color32::from_rgb(110, 99, 88))
        .stroke(egui::Stroke::new(3., egui::Color32::from_rgb(33, 22, 11)));
    let app = AZCharFourth::with_frame(f);

    eframe::run_native("AZChar-Fusion", options, Box::new(|_cc| Box::new(app)));
}

mod backend;
mod flow_control;
mod styles;
