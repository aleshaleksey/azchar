use crate::flow_control::connection::find_part;
use crate::flow_control::tables::{AttrValueKind, CharIdPack};
use crate::flow_control::*;
use crate::AZCharFourth;

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
                    Err(e) => println!("Error updating basics: {:?}", e),
                    Ok(dat) if !dat.is_empty() => {
                        let pack = CharIdPack::from_complete(char);
                        Self::update_attr_table(
                            AttrValueKind::Text,
                            pack,
                            dat,
                            &mut self.dbs,
                            char.attribute_map.as_mut().expect("Always set."),
                            &mut self.resources_basic,
                        );
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
                    Err(e) => println!("Error updating basics: {:?}", e),
                    Ok(dat) if !dat.is_empty() => {
                        let pack = CharIdPack::from_complete(char);
                        Self::update_attr_table(
                            AttrValueKind::Num,
                            pack,
                            dat,
                            &mut self.dbs,
                            char.attribute_map.as_mut().expect("Always set."),
                            &mut self.resources_points,
                        );
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
                                    println!("Part to update: {:?}", c);
                                    let pack = CharIdPack::from_part(char, c);
                                    Self::update_attr_table(
                                        AttrValueKind::Num,
                                        pack,
                                        d,
                                        &mut self.dbs,
                                        char.attribute_map.as_mut().expect("Always set."),
                                        &mut self.resources_body_hp,
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => println!("Error setting resources: {:?}", e),
                }
            });
        }
    }
}
