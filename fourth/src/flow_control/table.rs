use super::AZCharFourth;

use egui::Ui;

#[derive(Debug, Default)]
pub(super) struct Row {
    pub(super) title: String,
    pub(super) value: String,
    pub(super) label: String,
}

impl Row {
    pub(super) fn new(title: &str, value: &str) -> Self {
        Row {
            title: title.to_owned(),
            value: value.to_owned(),
            label: String::with_capacity(0),
        }
    }
    pub(super) fn with_label(title: &str, value: &str, label: &str) -> Self {
        Row {
            title: title.to_owned(),
            value: value.to_owned(),
            label: label.to_owned(),
        }
    }
}

impl AZCharFourth {
    pub(super) fn horizontal_table(
        ui: &mut Ui,
        values: &mut [Row],
        width: f32,
    ) -> Result<bool, String> {
        let width = width / values.len() as f32;
        let mut used = false;
        ui.horizontal(|ui| {
            for v in values.iter_mut() {
                ui.vertical(|ui| {
                    let _ = ui.selectable_label(false, &v.title);
                    let edit = egui::TextEdit::singleline(&mut v.value).desired_width(width);
                    let changed = ui.add(edit).changed();
                    if changed {
                        used = true;
                    }
                });
            }
        });
        Ok(used)
    }
}
