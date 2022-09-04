#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::expect_fun_call)]
use self::flow_control::FlowController;

fn main() {
    let mut options = eframe::NativeOptions::default();
    options.initial_window_size = Some(egui::Vec2::new(720., 768.));

    let f = styles::default_frame();
    let app = FlowController::with_frame(f);

    eframe::run_native("AZChar-Fusion", options, Box::new(|_cc| Box::new(app)));
}

pub(self) fn separator(ui: &mut egui::Ui) {
    ui.add(egui::Separator::default().spacing(3.));
}

mod backend;
mod file_dialog;
mod flow_control;
mod main_dialog;
mod styles;
