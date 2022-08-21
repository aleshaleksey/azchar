use crate::flow_control::*;
use crate::AZCharFourth;

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
        let mut reset = false;
        if !self.hidden_main_tables {
            ui.separator();
            ui.horizontal(|ui| {
                // Portrait or default for box.
                let portrait = self.images.get(&char.id()).unwrap_or(&self.default_img);
                {
                    let ib = egui::ImageButton::new(portrait.texture_id(ctx), [136., 136.]);
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
                    ui.separator();
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
                                } else {
                                    reset = true;
                                }
                            }
                            _ => {}
                        };
                    }
                    ui.separator();
                    {
                        let rows = &mut self.main_level_pro_table;
                        match AZCharFourth::horizontal_table(ui, rows, MAIN_W) {
                            Err(e) => println!("Error: {}", e),
                            Ok(true) => {
                                let res = AZCharFourth::update_attrs(&mut self.dbs, char, rows);
                                if let Err(e) = res {
                                    println!("Update level/proficiency: {:?}", e);
                                } else {
                                    // This is a special case for this system.
                                    if let Err(e) = update_all_proficiencies(&mut self.dbs, char) {
                                        println!("Can't update all: {:?}", e);
                                    };
                                    reset = true;
                                }
                            }
                            _ => {}
                        };
                    }
                    ui.separator();
                    {
                        let rows = &mut self.main_stat_table;
                        match AZCharFourth::horizontal_table(ui, rows, MAIN_W) {
                            Err(e) => println!("Error: {}", e),
                            Ok(true) => {
                                let res = AZCharFourth::update_attrs(&mut self.dbs, char, rows);
                                if let Err(e) = res {
                                    println!("Couldn't set image: {:?}", e);
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

// This is needed because we have a stupid ownership model.
fn update_all_proficiencies(
    dbs: &mut Option<LoadedDbs>,
    char: &mut CompleteCharacter,
) -> Result<(), String> {
    if let Some(ref mut dbs) = dbs {
        let char_key = (char.name.to_owned(), char.uuid().to_owned());
        let char_id = char.id().unwrap_or_default();
        let map = char.attribute_map.as_mut().expect("Always set.");
        let proficiency = map
            .get(&AttributeKey::new(PROFICIENCY.to_string(), char_id))
            .map(|v| v.value_num())
            .flatten();
        for (k, v) in map.iter_mut() {
            if k.key().contains("_proficiency") && k.key().contains("_skill_") {
                v.update_value_num_by_ref(proficiency);
                dbs.create_update_attribute(k.to_owned(), v.to_owned(), char_key.to_owned())?;
            }
        }
    }
    Ok(())
}
