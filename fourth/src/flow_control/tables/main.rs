use crate::flow_control::*;
use crate::AZCharFourth;

use eframe;
// use eframe::egui::Widget;

impl AZCharFourth {
    pub(crate) fn set_main_tables(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let char = self
            .current
            .as_mut()
            .expect("Do not call when no character.");
        if ui.selectable_label(false, "Basic Character Data").clicked() {
            self.hidden_main_tables = !self.hidden_main_tables;
        }
        if !self.hidden_main_tables {
            ui.horizontal(|ui| {
                // Portrait or default for box.
                let portrait = self.images.get(&char.id()).unwrap_or(&self.default_img);
                {
                    let ib = egui::ImageButton::new(portrait.texture_id(ctx), [128., 128.]);
                    if ib.ui(ui).clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("image", &["png", "jpg", "jpeg", "bmp"])
                            .pick_file()
                        {
                            println!("Picked: {:?}", path);
                            let name = char.name().to_owned();
                            let uuid = char.uuid().to_owned();
                            let id = char.id().unwrap();
                            let res = AZCharFourth::set_image(
                                &mut self.dbs,
                                &mut char.image,
                                &mut self.images,
                                name,
                                uuid,
                                id,
                                path,
                            );
                            if let Err(e) = res {
                                println!("Couldn't set image: {:?}", e);
                            }
                        } else {
                            println!("Failed to pick a file.");
                        }
                    }
                }
                // Set the three attribute tables.
                ui.vertical(|ui| {
                    {
                        let rows = &mut self.main_attr_table;
                        match AZCharFourth::horizontal_table(ui, rows, MAIN_W) {
                            Err(e) => println!("Error: {}", e),
                            Ok(true) => {
                                char.name = rows[0].value.to_owned();
                                if let Ok(n) = rows[1].value.parse() {
                                    char.speed = n;
                                }
                                if let Ok(n) = rows[2].value.parse() {
                                    char.weight = Some(n);
                                }
                                char.size = Some(rows[3].value.to_owned());
                                if let Ok(n) = rows[4].value.parse() {
                                    char.hp_current = Some(n);
                                }
                                if let Ok(n) = rows[5].value.parse() {
                                    char.hp_total = Some(n);
                                }
                                let part = char.to_bare_part();

                                let res = AZCharFourth::update_main(&mut self.dbs, part);
                                if let Err(e) = res {
                                    println!("Couldn't set image: {:?}", e);
                                }
                            }
                            _ => {}
                        };
                    }
                    {
                        let rows = &mut self.main_level_pro_table;
                        match AZCharFourth::horizontal_table(ui, rows, MAIN_W) {
                            Err(e) => println!("Error: {}", e),
                            Ok(true) => {
                                let res = AZCharFourth::update_attrs(&mut self.dbs, char, rows);
                                if let Err(e) = res {
                                    println!("Couldn't set image: {:?}", e);
                                }
                            }
                            _ => {}
                        };
                    }
                    {
                        let rows = &mut self.main_stat_table;
                        match AZCharFourth::horizontal_table(ui, rows, MAIN_W) {
                            Err(e) => println!("Error: {}", e),
                            Ok(true) => {
                                let res = AZCharFourth::update_attrs(&mut self.dbs, char, rows);
                                if let Err(e) = res {
                                    println!("Couldn't set image: {:?}", e);
                                }
                            }
                            _ => {}
                        };
                    }
                });
            });
        }
    }
}
