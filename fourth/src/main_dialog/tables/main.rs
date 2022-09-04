use crate::main_dialog::images::set_image;
use crate::main_dialog::AZCharFourth;
use crate::main_dialog::*;
use crate::separator;

impl AZCharFourth {
    pub(crate) fn set_main_tables(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let char = self
            .current
            .as_mut()
            .expect("Do not call when no character.");
        if ui.selectable_label(false, "Basic Character Data").clicked() {
            self.hidden_main_tables = !self.hidden_main_tables;
        }
        let mut reset = false;
        if !self.hidden_main_tables {
            separator(ui);
            let cid = char.id().expect("It's been through the DB.");
            ui.horizontal(|ui| {
                // Portrait or default for box.
                if let Err(e) = set_image(
                    &self.default_img,
                    ctx,
                    ui,
                    cid,
                    &mut self.images,
                    &mut self.file_dialog,
                ) {
                    error_dialog::fill(e, &mut self.error_dialog);
                };
                // Set the three attribute tables.
                ui.vertical(|ui| {
                    {
                        let rows = &mut self.main_attr_table;
                        match AZCharFourth::horizontal_table(ui, rows, MAIN_W) {
                            Err(e) => error_dialog::fill(e, &mut self.error_dialog),
                            Ok(true) => {
                                char.name = rows[0].val.to_owned();
                                if let Ok(n) = rows[1].val.parse() {
                                    char.speed = n;
                                }
                                if let Ok(n) = rows[2].val.parse() {
                                    char.weight = Some(n);
                                }
                                char.size = Some(rows[3].val.to_owned());
                                let part = char.to_bare_part();

                                let res = AZCharFourth::update_main(&mut self.dbs, part);
                                if let Err(e) = res {
                                    let e = format!("Couldn't set image: {:?}", e);
                                    error_dialog::fill(e, &mut self.error_dialog);
                                } else {
                                    reset = true;
                                }
                            }
                            _ => {}
                        };
                    }
                    separator(ui);
                    {
                        let rows = &mut self.main_level_pro_table;
                        match AZCharFourth::horizontal_table(ui, rows, MAIN_W * 0.75) {
                            Err(e) => println!("Error: {}", e),
                            Ok(true) => {
                                let res = AZCharFourth::update_attrs(&mut self.dbs, char, rows);
                                if let Err(e) = res {
                                    let e = format!("Update level/proficiency: {:?}", e);
                                    error_dialog::fill(e, &mut self.error_dialog);
                                } else {
                                    reset = true;
                                }
                                if let Ok(n) = rows[2].val.parse() {
                                    char.hp_current = Some(n);
                                }
                                if let Ok(n) = rows[3].val.parse() {
                                    char.hp_total = Some(n);
                                }

                                let part = char.to_bare_part();
                                let res = AZCharFourth::update_main(&mut self.dbs, part);
                                if let Err(e) = res {
                                    let e = format!("Couldn't set image: {:?}", e);
                                    error_dialog::fill(e, &mut self.error_dialog);
                                } else {
                                    reset = true;
                                }
                            }
                            _ => {}
                        };
                    }
                    separator(ui);
                    {
                        let rows = &mut self.main_stat_table;
                        match AZCharFourth::horizontal_table(ui, rows, MAIN_W * 0.75) {
                            Err(e) => println!("Error: {}", e),
                            Ok(true) => {
                                let res = AZCharFourth::update_attrs(&mut self.dbs, char, rows);
                                if let Err(e) = res {
                                    let e = format!("Couldn't set image: {:?}", e);
                                    error_dialog::fill(e, &mut self.error_dialog);
                                } else {
                                    reset = true;
                                }
                            }
                            _ => {}
                        };
                    }
                });
            });

            if reset {
                let char = std::mem::take(&mut self.current).expect("Is here.");
                if let Err(e) = self.set_character(char) {
                    println!("Couldn't reset character: {:?}", e);
                } else {
                    println!("Char updated and reset.");
                }
            }
        }
    }
}