use crate::styles;
use crate::AZCharFourth;

impl AZCharFourth {
    pub(super) fn set_error_dialog(&mut self, ctx: &egui::Context) {
        let mut hide = false;
        if let Some(cont) = self.error_dialog.as_ref() {
            egui::Area::new("error-details")
                .default_pos(egui::pos2(100.0, 100.0))
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    ui.set_style(styles::style());
                    self.frame.show(ui, |ui| {
                        ui.vertical(|ui| {
                            let _ = ui.selectable_label(false, cont).clicked();
                            ui.horizontal(|ui| {
                                let a = ui.button("Ok").clicked();
                                let b = ui.button("Not Ok").clicked();
                                if a || b {
                                    hide = true;
                                }
                            })
                        })
                    });
                });
        }
        if hide {
            self.error_dialog = None;
        }
    }
}

pub(super) fn fill<D: std::fmt::Debug>(error: D, error_container: &mut Option<String>) {
    *error_container = Some(format!("{:?}", error));
}
