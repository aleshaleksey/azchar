use crate::main_dialog::connection::find_part;
use crate::main_dialog::error_dialog;
use crate::main_dialog::tables::{AttrValueKind, CharIdPack};
use crate::main_dialog::AZCharFourth;
use crate::main_dialog::*;
use crate::separator;

// use eframe::egui::Widget;

impl AZCharFourth {
    pub(crate) fn set_resource_tables(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        let char = self
            .current
            .as_mut()
            .expect("Do not call when no character.");

        if ui.selectable_label(false, "Character Resource").clicked() {
            self.hidden_resource_tables = !self.hidden_resource_tables;
        }
        if !self.hidden_resource_tables {
            separator(ui);
            ui.horizontal(|ui| {
                match self.resources_basic.set_attr_based_resource(
                    "",
                    ui,
                    (68., 75.),
                    AttrValueKind::Text,
                ) {
                    Err(e) => error_dialog::fill(e, &mut self.error_dialog),
                    Ok(dat) if !dat.is_empty() => {
                        let pack = CharIdPack::from_complete(char);
                        if let Err(e) = Self::update_attr_table(
                            pack,
                            dat,
                            &mut self.dbs,
                            char.attribute_map.as_mut().expect("Always set."),
                            &mut self.resources_basic,
                        ) {
                            error_dialog::fill(e, &mut self.error_dialog);
                        };
                    }
                    _ => {}
                }
                separator(ui);
                match self.resources_points.set_attr_based_resource(
                    "",
                    ui,
                    (60., 40.),
                    AttrValueKind::Num,
                ) {
                    Err(e) => error_dialog::fill(e, &mut self.error_dialog),
                    Ok(dat) if !dat.is_empty() => {
                        let pack = CharIdPack::from_complete(char);
                        if let Err(e) = Self::update_attr_table(
                            pack,
                            dat,
                            &mut self.dbs,
                            char.attribute_map.as_mut().expect("Always set."),
                            &mut self.resources_points,
                        ) {
                            error_dialog::fill(e, &mut self.error_dialog);
                        };
                    }
                    _ => {}
                }
                separator(ui);
                match self.resources_body_hp.set_attr_based_resource(
                    "",
                    ui,
                    (70., 40.),
                    AttrValueKind::Num,
                ) {
                    Ok(dat) => {
                        for i in 0..self.resources_body_hp.row_labels.len() {
                            // We use visible here because we index parts by `visible`.
                            let key = &self.resources_body_hp.row_labels[i].visible;
                            let d = dat
                                .iter()
                                .filter(|(r_idx, _)| *r_idx == i)
                                .copied()
                                .collect::<Vec<_>>();
                            if !d.is_empty() {
                                if let Some(c) = find_part(char, key) {
                                    let pack = CharIdPack::from_part(char, c);
                                    if let Err(e) = Self::update_attr_table(
                                        pack,
                                        d,
                                        &mut self.dbs,
                                        char.attribute_map.as_mut().expect("Always set."),
                                        &mut self.resources_body_hp,
                                    ) {
                                        error_dialog::fill(e, &mut self.error_dialog);
                                    };
                                }
                            }
                        }
                    }
                    Err(e) => error_dialog::fill(e, &mut self.error_dialog),
                }
            });
        }
    }
}
