use crate::flow_control::images::set_image;
use crate::flow_control::*;
use crate::AZCharFourth;

use azchar_database::character::attribute::{InputAttribute};
use azchar_database::character::character::{InputCharacter};
use azchar_database::shared::Part;

use std::sync::Mutex;

const LABEL_SIZE: [f32; 2] = [200., 21.];
const SMALL_LABEL_SIZE: [f32; 2] = [60., 21.];

impl AZCharFourth {
    pub(crate) fn set_parts(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if ui.selectable_label(false, "Character Attacks").clicked() {
            self.hidden_attacks = !self.hidden_attacks;
        }
        if !self.hidden_attacks {
            separator(ui);
            if let Err(e) = self.set_parts_list(ui, Part::Ability, Some("attack")) {
                println!("We have a problem: {:?}", e);
            };
        }
        separator(ui);

        if ui.selectable_label(false, "Character Specials").clicked() {
            self.hidden_specials = !self.hidden_specials;
        }
        if !self.hidden_specials {
            separator(ui);
            if let Err(e) = self.set_parts_list(ui, Part::Ability, Some("specials")) {
                println!("We have a problem: {:?}", e);
            };
        }
        separator(ui);

        if ui.selectable_label(false, "Character Inventory").clicked() {
            self.hidden_inventory = !self.hidden_inventory;
        }
        if !self.hidden_inventory {
            separator(ui);
            if let Err(e) = self.set_parts_list(ui, Part::InventoryItem, None) {
                println!("We have a problem: {:?}", e);
            };
        }
        separator(ui);

        if ui.selectable_label(false, "Character Spells").clicked() {
            self.hidden_spells = !self.hidden_spells;
        }
        if !self.hidden_spells {
            separator(ui);
            if let Err(e) = self.set_parts_list(ui, Part::Ability, Some("spell")) {
                println!("We have a use std::sync::Mutex;problem: {:?}", e);
            };
        }
        if let PartOption::Existing(id) = self.part_window {
            if let Err(e) = self.display_part_details(ui, id, ctx) {
                println!("Part detail error: {:?}", e);
            }
        }
    }

    /// A generic function to set a list of parts.
    fn set_parts_list(
        &mut self,
        ui: &mut egui::Ui,
        part_type: Part,
        character_type: Option<&str>,
    ) -> Result<(), String> {
        let char = self
            .current
            .as_mut()
            .expect("Do not call when no character.");
        let dbs = self
            .dbs
            .as_mut()
            .expect("We have a DB because we have a character.");

        let mut counter = 0;
        ui.vertical(|ui| {
            let new_clicked = ui.selectable_label(false, "Add New").clicked();
            // If we have a new character to add, it must conform to what we have.
            if self.part_window.is_none() && new_clicked {
                let mut new_input = InputCharacter::default();
                new_input.part_type = part_type;
                if let Some(t) = character_type {
                    new_input.character_type = t.to_owned();
                }
                new_input.belongs_to = char.id();
                self.part_window = PartOption::New(new_input);
            } else if self.part_window.is_new() && new_clicked {
                self.part_window = PartOption::None;
            }
            if let Err(e) =
                self.part_window
                    .set_and_update_new_part((part_type, character_type), char, dbs, ui)
            {
                println!("Error in new part: {:?}", e);
            }
            for i in (0..char.parts.len()).rev() {
                let p = &mut char.parts[i];
                // Determine whether this part gets set or not.
                if p.part_type() != part_type
                    || character_type
                        .map(|t| t != p.character_type())
                        .unwrap_or(false)
                {
                    continue;
                }
                ui.horizontal(|ui| {
                    // Label
                    let id = counter.to_string();
                    let id = egui::SelectableLabel::new(false, &id);
                    let id = ui.add_sized([30., 21.], id).clicked();
                    // name.
                    let name = egui::SelectableLabel::new(false, p.name());
                    let name = ui.add_sized([204., 21.], name).clicked();
                    // Weight.
                    if p.weight.is_none() {
                        p.weight = Some(0);
                    };
                    let w = p.weight.as_ref().expect("It is some.").to_string();
                    let w = egui::SelectableLabel::new(false, w);
                    let w = ui.add(w).clicked();
                    //s
                    let kind = egui::SelectableLabel::new(false, p.character_type());
                    let kind = ui.add(kind).clicked();

                    if self.part_window.is_none() && (name || id || w || kind) {
                        let id = p.id().unwrap_or_default();
                        self.part_window = PartOption::Existing(id);
                        let mut attr = InputAttribute::default();
                        attr.of = id;
                        self.attr_option = Some(attr);
                    } else if (name || id || w) && self.part_window.is_existing() {
                        self.part_window = PartOption::None;
                    }
                });
                counter += 1;
            }
        });
        Ok(())
    }

    fn display_part_details(
        &mut self,
        _ui: &mut egui::Ui,
        part_id: i64,
        ctx: &egui::Context,
    ) -> Result<(), String> {
        egui::Area::new("part-details")
            .default_pos(egui::pos2(32.0, 32.0))
            .show(ctx, |ui| {
                ui.set_style(styles::style());
                self.frame.show(ui, |ui| {
                    // First set the parts details: NB we do not need things like speed/Weight
                    // for abilities.
                    let part = self
                        .current
                        .as_ref()
                        .expect("`current` is real.")
                        .parts
                        .iter()
                        .find(|p| p.id() == Some(part_id))
                        .expect("It must be there (borrow checker hates me).");

                    let pid: i64 = part.id().expect("It's been through the DB.");
                    let part_type: Part = part.part_type();
                    let character_type = part.character_type().to_string();

                    ui.vertical(|ui| {
                        let dbs = self.dbs.as_mut().expect("We could not get here otherwise.");

                        let char = self.current.as_ref().expect("There's a character here.");
                        let char_key = (char.name.to_owned(), char.uuid().to_owned());

                        ui.horizontal(|ui| {
                            let part = Mutex::new(
                                self.current
                                    .as_mut()
                                    .expect("`current` is real.")
                                    .parts
                                    .iter_mut()
                                    .find(|p| p.id() == Some(part_id))
                                    .expect("It must be there (borrow checker hates me)."),
                            );
                            // Portrait or default for box.
                            if let Ok(mut part) = part.lock() {
                                set_image(
                                    &self.default_img,
                                    ctx,
                                    ui,
                                    dbs,
                                    &mut part.image,
                                    char_key.clone(),
                                    pid,
                                    &mut self.images,
                                );
                            }

                            ui.vertical(|ui| {
                                // First the part name.
                                ui.horizontal(|ui| {
                                    let label = format!("{} name:", part_type.to_string());
                                    let l = egui::SelectableLabel::new(false, &label);
                                    let _ = ui.add_sized(LABEL_SIZE, l).clicked();
                                    //
                                    if let Ok(mut part) = part.lock() {
                                        let l = egui::TextEdit::singleline(&mut part.name);
                                        if ui.add_sized(LABEL_SIZE, l).changed() {
                                            if let Err(e) = dbs.create_update_part(
                                                part.to_owned(),
                                                char_key.to_owned(),
                                            ) {
                                                println!("Key: {:?}", char_key);
                                                println!("Update error: {:?}", e);
                                            }
                                        };
                                    }
                                });
                                // Then the part subtype.
                                ui.horizontal(|ui| {
                                    let l = egui::SelectableLabel::new(false, "Subtype:");
                                    let _ = ui.add_sized(LABEL_SIZE, l).clicked();
                                    //
                                    let l = egui::Label::new(&character_type);
                                    let _ = ui.add_sized(LABEL_SIZE, l).clicked();
                                    Ok::<(), String>(())
                                });
                                // Abilities do not have physical attributes.
                                if !matches!(part_type, Part::Ability) {
                                    // Speed.
                                    ui.horizontal(|ui| {
                                        let l = egui::SelectableLabel::new(false, "Speed");
                                        let _ = ui.add_sized(LABEL_SIZE, l).clicked();
                                        if let Ok(mut part) = part.lock() {
                                            let mut speed = part.speed.to_string();
                                            let l = egui::TextEdit::singleline(&mut speed);
                                            if ui.add_sized(LABEL_SIZE, l).changed() {
                                                part.speed =
                                                    speed.parse::<i32>().unwrap_or(part.speed);
                                                if let Err(e) = dbs.create_update_part(
                                                    part.to_owned(),
                                                    char_key.to_owned(),
                                                ) {
                                                    println!("Key: {:?}", char_key);
                                                    println!("Update error: {:?}", e);
                                                }
                                            }
                                        };
                                        Ok::<(), String>(())
                                    });
                                    // Weight.
                                    ui.horizontal(|ui| {
                                        let l = egui::SelectableLabel::new(false, "Weight");
                                        let _ = ui.add_sized(LABEL_SIZE, l).clicked();

                                        if let Ok(mut part) = part.lock() {
                                            let mut w = part.weight.unwrap_or(0).to_string();
                                            let l = egui::TextEdit::singleline(&mut w);
                                            if ui.add_sized(LABEL_SIZE, l).changed() {
                                                if let Ok(r) = w.parse::<i32>() {
                                                    part.weight = Some(r);
                                                    if let Err(e) = dbs.create_update_part(
                                                        part.to_owned(),
                                                        char_key.to_owned(),
                                                    ) {
                                                        println!("Key: {:?}", char_key);
                                                        println!("Update error: {:?}", e);
                                                    }
                                                }
                                            }
                                        };
                                        Ok::<(), String>(())
                                    });
                                    // Size.
                                    ui.horizontal(|ui| {
                                        let l = egui::SelectableLabel::new(false, "Size");
                                        let _ = ui.add_sized(LABEL_SIZE, l).clicked();

                                        if let Ok(mut part) = part.lock() {
                                            let backup_string = String::new();
                                            let mut s = part
                                                .size
                                                .as_ref()
                                                .unwrap_or(&backup_string)
                                                .to_string();
                                            let l = egui::TextEdit::singleline(&mut s);
                                            if ui.add_sized(LABEL_SIZE, l).changed() {
                                                part.size = Some(s);
                                                if let Err(e) = dbs.create_update_part(
                                                    part.to_owned(),
                                                    char_key.to_owned(),
                                                ) {
                                                    println!("Key: {:?}", char_key);
                                                    println!("Update error: {:?}", e);
                                                }
                                            }
                                        };
                                        Ok::<(), String>(())
                                    });
                                    // HP.
                                    ui.horizontal(|ui| {
                                        let l = egui::SelectableLabel::new(false, "HP total");
                                        let _ = ui.add_sized(LABEL_SIZE, l).clicked();

                                        if let Ok(mut part) = part.lock() {
                                            let mut w = part.hp_total.unwrap_or(0).to_string();
                                            let l = egui::TextEdit::singleline(&mut w);
                                            if ui.add_sized(LABEL_SIZE, l).changed() {
                                                if let Ok(r) = w.parse::<i32>() {
                                                    part.hp_total = Some(r);
                                                    if let Err(e) = dbs.create_update_part(
                                                        part.to_owned(),
                                                        char_key.to_owned(),
                                                    ) {
                                                        println!("Key: {:?}", char_key);
                                                        println!("Update error: {:?}", e);
                                                    }
                                                }
                                            }
                                        };
                                        Ok::<(), String>(())
                                    });
                                    ui.horizontal(|ui| {
                                        separator(ui);
                                        let l = egui::SelectableLabel::new(false, "HP current");
                                        let _ = ui.add_sized(LABEL_SIZE, l).clicked();

                                        if let Ok(mut part) = part.lock() {
                                            let mut w = part.hp_current.unwrap_or(0).to_string();
                                            let l = egui::TextEdit::singleline(&mut w);
                                            if ui.add_sized(LABEL_SIZE, l).changed() {
                                                if let Ok(r) = w.parse::<i32>() {
                                                    part.hp_current = Some(r);
                                                    if let Err(e) = dbs.create_update_part(
                                                        part.to_owned(),
                                                        char_key.to_owned(),
                                                    ) {
                                                        println!("Key: {:?}", char_key);
                                                        println!("Update error: {:?}", e);
                                                    }
                                                }
                                            }
                                        };
                                        Ok::<(), String>(())
                                    });
                                }
                            });
                            // End of basics vertical.
                        });
                        // Insert Blurb && Insert Attributes.
                        ui.vertical(|ui| {
                            {
                                let part = Mutex::new(
                                    self.current
                                        .as_mut()
                                        .expect("`current` is real.")
                                        .parts
                                        .iter_mut()
                                        .find(|p| p.id() == Some(part_id))
                                        .expect("It must be there (borrow checker hates me)."),
                                );

                                if let Ok(mut part) = part.lock() {
                                    if let Some((k, v)) =
                                        part.attributes.iter_mut().find(|a| a.0.key() == "Blurb")
                                    {
                                        if v.value_text().as_ref().is_none() {
                                            v.update_value_text_by_ref(Some(String::new()));
                                        }
                                        let mut content =
                                            v.value_text().as_ref().expect("Is some").to_owned();
                                        let width = 2. * LABEL_SIZE[0] + SMALL_LABEL_SIZE[0];
                                        let note_edit = egui::TextEdit::multiline(&mut content)
                                            .frame(true)
                                            .margin(egui::Vec2::new(2., 2.))
                                            .desired_rows(10)
                                            .desired_width(width)
                                            .lock_focus(true)
                                            .cursor_at_end(true);
                                        if ui.add_sized([500., 100.], note_edit).changed() {
                                            v.update_value_text_by_ref(Some(content));
                                            if let Err(e) = dbs.create_update_attribute(
                                                k.to_owned(),
                                                v.to_owned(),
                                                char_key.to_owned(),
                                            ) {
                                                println!("Key: {:?}", char_key);
                                                println!("Update error: {:?}", e);
                                            }
                                        }
                                    }
                                }
                                ui.horizontal(|ui| {
                                    let l = egui::SelectableLabel::new(false, "Attribute");
                                    let _ = ui.add_sized(LABEL_SIZE, l).clicked();
                                    //
                                    let l = egui::SelectableLabel::new(false, "Num Value");
                                    let _ = ui.add_sized(SMALL_LABEL_SIZE, l).clicked();
                                    //
                                    let l = egui::SelectableLabel::new(false, "Text Value");
                                    let _ = ui.add_sized(LABEL_SIZE, l).clicked();
                                });
                                egui::ScrollArea::vertical()
                                    .max_height(LABEL_SIZE[1] * 10.)
                                    .show(ui, |ui| {
                                        if let Ok(mut part) = part.lock() {
                                            for (k, v) in part
                                                .attributes
                                                .iter_mut()
                                                .filter(|(k, _)| k.key() != "Blurb")
                                            {
                                                ui.horizontal(|ui| {
                                                    // Deal with the label.
                                                    let label = k
                                                        .key()
                                                        .split("_")
                                                        .last()
                                                        .unwrap_or_default();
                                                    let l =
                                                        egui::SelectableLabel::new(false, label);
                                                    let _ = ui.add_sized(LABEL_SIZE, l).clicked();
                                                    // Deal with the numerical value.
                                                    let mut w =
                                                        v.value_num().unwrap_or(0).to_string();
                                                    let l = egui::TextEdit::singleline(&mut w);
                                                    if ui.add_sized(SMALL_LABEL_SIZE, l).changed() {
                                                        if let Ok(r) = w.parse::<i64>() {
                                                            v.update_value_num_by_ref(Some(r));
                                                            if let Err(e) = dbs
                                                                .create_update_attribute(
                                                                    k.to_owned(),
                                                                    v.to_owned(),
                                                                    char_key.to_owned(),
                                                                )
                                                            {
                                                                println!("Key: {:?}", char_key);
                                                                println!("Update error: {:?}", e);
                                                            }
                                                        }
                                                    };
                                                    // Deal with the string value.
                                                    let default = String::new();
                                                    let mut w = v
                                                        .value_text()
                                                        .as_ref()
                                                        .unwrap_or(&default)
                                                        .to_string();
                                                    let l = egui::TextEdit::singleline(&mut w);
                                                    if ui.add_sized(LABEL_SIZE, l).changed() {
                                                        v.update_value_text_by_ref(Some(w));
                                                        if let Err(e) = dbs.create_update_attribute(
                                                            k.to_owned(),
                                                            v.to_owned(),
                                                            char_key.to_owned(),
                                                        ) {
                                                            println!("Key: {:?}", char_key);
                                                            println!("Update error: {:?}", e);
                                                        }
                                                    };
                                                    Ok::<(), String>(())
                                                });
                                            }
                                        }
                                    });
                            }
                            // End of attribute list.
                            if let Some(ref mut new_attr) = self.attr_option {
                                let displayed_key = new_attr
                                    .key
                                    .split('_')
                                    .last()
                                    .unwrap_or_default()
                                    .to_owned();

                                ui.horizontal(|ui| {
                                    let part = Mutex::new(
                                        self.current
                                            .as_mut()
                                            .expect("`current` is real.")
                                            .parts
                                            .iter_mut()
                                            .find(|p| p.id() == Some(part_id))
                                            .expect("It must be there (borrow checker hates me)."),
                                    );
                                    egui::ComboBox::from_label("")
                                        .width(LABEL_SIZE[0])
                                        .selected_text(displayed_key)
                                        .show_ui(ui, |ui| {
                                            if let Ok(part) = part.lock() {
                                                let part_attrs = &part.attributes;
                                                let permitted =
                                                    dbs.permitted_attrs.iter().filter(|pa| {
                                                        part_attrs
                                                            .iter()
                                                            .all(|x| x.0.key() != &pa.key)
                                                            && pa
                                                                .part_name
                                                                .as_ref()
                                                                .map(|x| x == &character_type)
                                                                .unwrap_or(true)
                                                            && pa
                                                                .part_type
                                                                .as_ref()
                                                                .map(|x| *x == part_type)
                                                                .unwrap_or(true)
                                                    });
                                                for v in permitted {
                                                    let displayed_tail = v
                                                        .key
                                                        .split('_')
                                                        .last()
                                                        .unwrap_or_default()
                                                        .to_owned();
                                                    ui.selectable_value(
                                                        &mut new_attr.key,
                                                        v.key.to_owned(),
                                                        displayed_tail,
                                                    );
                                                }
                                            }
                                        });

                                    if let Some(p) = dbs.permitted_attrs.iter().find(|pa| {
                                        (&pa.key == &new_attr.key)
                                            && pa
                                                .part_name
                                                .as_ref()
                                                .map(|x| x == &character_type)
                                                .unwrap_or(true)
                                            && pa
                                                .part_type
                                                .as_ref()
                                                .map(|x| *x == part_type)
                                                .unwrap_or(true)
                                    }) {
                                        new_attr.description =
                                            Some(p.attribute_description.to_owned());
                                        let l = egui::SelectableLabel::new(
                                            false,
                                            &p.attribute_description,
                                        );
                                        let _ = ui.add_sized(LABEL_SIZE, l).clicked();
                                    }
                                    if ui.button("Add Attribute").clicked() {
                                        match dbs.create_attribute(
                                            new_attr.to_owned(),
                                            char_key.to_owned(),
                                        ) {
                                            Err(e) => println!("Couldn't add attribute: {:?}", e),
                                            Ok(mut c) => {
                                                c.create_attribute_map();
                                                let char = self.current.as_mut().expect("Exists");
                                                *char = c;
                                            }
                                        }
                                    }
                                });
                            }
                        });
                        // End of basics and image horizontal.
                    });
                    // End of All hope.
                });
                Ok::<(), String>(())
            })
            .inner?;
        Ok(())
    }
}
