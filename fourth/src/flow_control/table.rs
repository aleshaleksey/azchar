use super::AZCharFourth;

use egui::Ui;
const D20_SKILL: &str = "d20_skill_";

#[derive(Debug, Default)]
pub(super) struct Row {
    pub(super) title: String,
    pub(super) value: String,
    pub(super) label: String,
}

#[derive(Debug, Clone, Default)]
pub(super) struct Label {
    pub(super) visible: String,
    pub(super) key: String,
}

impl Label {
    pub(super) fn new(visible: &str, key: &str) -> Self {
        Self {
            visible: visible.to_owned(),
            key: key.to_owned(),
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct DynamicTable {
    // Labels determine column labels and no. of columns.
    pub(super) column_labels: Vec<Label>,
    // Labels determine row labels and no. of rows.
    pub(super) row_labels: Vec<Label>,
    // Rows is slow, Columns is fast. (Counterintuitive but comfortable)
    pub(super) cells: Vec<Vec<String>>,
}

impl DynamicTable {
    pub(super) fn empty_with_row_column_counts(r: usize, c: usize) -> Self {
        let column_labels = std::iter::repeat(Label::default())
            .take(c)
            .collect::<Vec<_>>();
        let row_labels = std::iter::repeat(Label::default())
            .take(r)
            .collect::<Vec<_>>();
        let mut cells = Vec::new();
        for _ in 0..c {
            cells.push(
                std::iter::repeat(String::default())
                    .take(r)
                    .collect::<Vec<_>>(),
            );
        }
        DynamicTable {
            column_labels,
            row_labels,
            cells,
        }
    }

    pub(super) fn add_column_labels(&mut self, cl: Vec<Label>) {
        debug_assert!(self.column_labels.is_empty(), "Column labels not empty.");
        debug_assert!(self.cells.is_empty(), "Row cells not empty.");
        debug_assert!(self.row_labels.is_empty(), "Row labels not empty.");
        self.column_labels = cl;
    }

    pub(super) fn add_row_with_label(&mut self, rl: Label, row: Vec<String>) {
        debug_assert_eq!(row.len(), self.column_labels.len());
        self.row_labels.push(rl);
        self.cells.push(row);
    }

    pub(super) fn d100_skill_table(
        &mut self,
        ui: &mut Ui,
        width: f32,
    ) -> Result<Vec<(usize, usize)>, String> {
        let w = width / (1. + self.column_labels.len() as f32);
        let mut used = Vec::new();
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.selectable_label(false, "D100 SKILLS");
                for l in self.column_labels.iter() {
                    ui.selectable_label(false, &l.visible);
                }
            });
            for (r_idx, (rl, mut row)) in self
                .row_labels
                .iter()
                .zip(self.cells.iter_mut())
                .enumerate()
            {
                ui.horizontal(|ui| {
                    ui.selectable_label(false, &rl.visible);
                    for (c_idx, mut r) in row.iter_mut().enumerate() {
                        let edit = egui::TextEdit::singleline(r).desired_width(w);
                        let changed = ui.add_sized([w, 21.], edit).changed();
                        if changed {
                            used.push((r_idx, c_idx));
                        }
                    }
                });
            }
        });
        Ok(used)
    }

    pub(super) fn d20_skill_table(
        &mut self,
        ui: &mut Ui,
        width: f32,
    ) -> Result<Vec<(usize, usize)>, String> {
        let w = width / (1. + self.column_labels.len() as f32);
        let mut used = Vec::new();
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.selectable_label(false, "D20 SKILLS");
                for l in self.column_labels.iter() {
                    ui.selectable_label(false, &l.visible);
                }
            });
            for (r_idx, (rl, mut row)) in self
                .row_labels
                .iter()
                .zip(self.cells.iter_mut())
                .enumerate()
            {
                ui.horizontal(|ui| {
                    ui.selectable_label(false, &rl.visible);
                    ui.selectable_label(false, &row[0]); // GOV
                    {
                        // BOX
                        let edit = egui::RadioButton::new(row[1] != "0", "");
                        let changed = ui.add_sized([w, 21.], edit).changed();
                        if changed {
                            used.push((r_idx, 1));
                        }
                    }
                    ui.selectable_label(false, &rl.visible);
                    for (c_idx, mut r) in row.iter_mut().enumerate().skip(2) {
                        let edit = egui::TextEdit::singleline(r).desired_width(w);
                        let changed = ui.add_sized([w, 21.], edit).changed();
                        if changed {
                            used.push((r_idx, c_idx));
                        }
                    }
                });
            }
        });
        Ok(used)
    }
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
                    let changed = ui.add_sized([width, 21.], edit).changed();
                    if changed {
                        used = true;
                    }
                });
            }
        });
        Ok(used)
    }
}
