use crate::flow_control::images::set_image;
use crate::flow_control::*;
use crate::AZCharFourth;

use azchar_database::character::character::InputCharacter;
use azchar_database::character::*;
use azchar_database::shared::Part;

type Key<'a> = (Part, Option<&'a str>);

fn compare_keys<'a>(gui_part: Key<'a>, new_type: Part, new_character_type: &str) -> bool {
    if gui_part.0 == new_type && gui_part.1.map(|t| t == new_character_type).unwrap_or(true) {
        true
    } else {
        false
    }
}

// use eframe::egui::Widget;
#[derive(Clone, Debug)]
pub(crate) enum PartOption {
    Existing(i64),
    New(InputCharacter),
    None,
}

impl PartOption {
    pub(crate) fn is_none(&self) -> bool {
        matches!(self, &Self::None)
    }
    pub(crate) fn is_new(&self) -> bool {
        matches!(self, &Self::New(_))
    }
    pub(crate) fn is_existing(&self) -> bool {
        matches!(self, &Self::Existing(_))
    }
    fn set_and_update_new_part(
        &mut self,
        key: Key,
        char: &mut CompleteCharacter,
        dbs: &mut LoadedDbs,
        ui: &mut egui::Ui,
    ) -> Result<(), String> {
        let exit = match self {
            Self::New(new) if compare_keys(key, new.part_type, &new.character_type) => {
                ui.horizontal(|ui| {
                    // Set the first row with the name.
                    ui.vertical(|ui| {
                        ui.add(egui::SelectableLabel::new(false, "Name"));
                        ui.add(egui::TextEdit::singleline(&mut new.name));
                    });
                    // Set the second row with the character kind selection.
                    ui.vertical(|ui| {
                        ui.add(egui::SelectableLabel::new(false, "Kind"));
                        let permitted_character_types = match key.1 {
                            Some(k) => vec![k],
                            None => filter_parts(dbs, new),
                        };
                        let pt = &mut new.character_type;
                        egui::ComboBox::from_label("")
                            .selected_text(format!("{:?}", pt))
                            .show_ui(ui, |ui| {
                                for v in permitted_character_types {
                                    ui.selectable_value(pt, v.to_string(), v);
                                }
                            });
                    });
                    // Set the create new part button.
                    ui.vertical(|ui| {
                        ui.add(egui::Label::new(""));
                        if ui.button("Create").clicked() {
                            match dbs.create_part(
                                new.to_owned(),
                                (char.name().to_owned(), char.uuid().to_owned()),
                            ) {
                                Err(e) => println!("Error updating note: {:?}", e),
                                Ok(updated_char) => {
                                    *char = updated_char;
                                    char.create_attribute_map();
                                    return true;
                                }
                            }
                        };
                        false
                    })
                })
                .inner
                .inner
            }
            _ => false,
        };
        if exit {
            *self = Self::None;
        }
        Ok(())
    }
}

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
                println!("We have a problem: {:?}", e);
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
        const LABEL_SIZE: [f32; 2] = [200., 21.];
        const SMALL_LABEL_SIZE: [f32; 2] = [60., 21.];
        egui::Area::new("part-details")
            .default_pos(egui::pos2(32.0, 32.0))
            .show(ctx, |ui| {
                ui.set_style(styles::style());
                crate::styles::default_frame().show(ui, |ui| {
                    let char = self.current.as_ref().expect("There's a character here.");
                    let key = (char.name.to_owned(), char.uuid().to_owned());

                    let dbs = self.dbs.as_mut().expect("We could not get here otherwise.");
                    let part = self
                        .current
                        .as_mut()
                        .expect("`current` is real.")
                        .parts
                        .iter_mut()
                        .find(|p| p.id() == Some(part_id))
                        .expect("It must be there (borrow checker hates me).");

                    // First set the parts details: NB we do not need things like speed/Weight
                    // for abilities.
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            let pid = part.id().expect("It's been through the DB.");
                            // Portrait or default for box.
                            set_image(
                                &self.default_img,
                                ctx,
                                ui,
                                dbs,
                                &mut part.image,
                                key.clone(),
                                pid,
                                &mut self.images,
                            );

                            ui.vertical(|ui| {
                                // First the part name.
                                ui.horizontal(|ui| {
                                    let label = format!("{} name:", part.part_type().to_string());
                                    let l = egui::SelectableLabel::new(false, &label);
                                    let _ = ui.add_sized(LABEL_SIZE, l).clicked();
                                    //
                                    let l = egui::TextEdit::singleline(&mut part.name);
                                    if ui.add_sized(LABEL_SIZE, l).changed() {
                                        if let Err(e) =
                                            dbs.create_update_part(part.to_owned(), key.to_owned())
                                        {
                                            println!("Key: {:?}", key);
                                            println!("Update error: {:?}", e);
                                        }
                                    };
                                    Ok::<(), String>(())
                                });
                                // Then the part subtype.
                                ui.horizontal(|ui| {
                                    let l = egui::SelectableLabel::new(false, "Subtype:");
                                    let _ = ui.add_sized(LABEL_SIZE, l).clicked();
                                    //
                                    let l = egui::Label::new(part.character_type());
                                    let _ = ui.add_sized(LABEL_SIZE, l).clicked();
                                    Ok::<(), String>(())
                                });
                                // Abilities do not have physical attributes.
                                if !matches!(part.part_type(), Part::Ability) {
                                    // Speed.
                                    ui.horizontal(|ui| {
                                        let l = egui::SelectableLabel::new(false, "Speed");
                                        let _ = ui.add_sized(LABEL_SIZE, l).clicked();

                                        let mut speed = part.speed.to_string();
                                        let l = egui::TextEdit::singleline(&mut speed);
                                        if ui.add_sized(LABEL_SIZE, l).changed() {
                                            part.speed = speed.parse::<i32>().unwrap_or(part.speed);
                                            if let Err(e) = dbs
                                                .create_update_part(part.to_owned(), key.to_owned())
                                            {
                                                println!("Key: {:?}", key);
                                                println!("Update error: {:?}", e);
                                            }
                                        };
                                        Ok::<(), String>(())
                                    });
                                    // Weight.
                                    ui.horizontal(|ui| {
                                        let l = egui::SelectableLabel::new(false, "Weight");
                                        let _ = ui.add_sized(LABEL_SIZE, l).clicked();

                                        let mut w = part.weight.unwrap_or(0).to_string();
                                        let l = egui::TextEdit::singleline(&mut w);
                                        if ui.add_sized(LABEL_SIZE, l).changed() {
                                            if let Ok(r) = w.parse::<i32>() {
                                                part.weight = Some(r);
                                                if let Err(e) = dbs.create_update_part(
                                                    part.to_owned(),
                                                    key.to_owned(),
                                                ) {
                                                    println!("Key: {:?}", key);
                                                    println!("Update error: {:?}", e);
                                                }
                                            }
                                        };
                                        Ok::<(), String>(())
                                    });
                                    // Size.
                                    ui.horizontal(|ui| {
                                        let l = egui::SelectableLabel::new(false, "Size");
                                        let _ = ui.add_sized(LABEL_SIZE, l).clicked();

                                        let backup_string = String::new();
                                        let mut s = part
                                            .size
                                            .as_ref()
                                            .unwrap_or(&backup_string)
                                            .to_string();
                                        let l = egui::TextEdit::singleline(&mut s);
                                        if ui.add_sized(LABEL_SIZE, l).changed() {
                                            part.size = Some(s);
                                            if let Err(e) = dbs
                                                .create_update_part(part.to_owned(), key.to_owned())
                                            {
                                                println!("Key: {:?}", key);
                                                println!("Update error: {:?}", e);
                                            }
                                        };
                                        Ok::<(), String>(())
                                    });
                                    // HP.
                                    ui.horizontal(|ui| {
                                        let l = egui::SelectableLabel::new(false, "HP total");
                                        let _ = ui.add_sized(LABEL_SIZE, l).clicked();

                                        let mut w = part.hp_total.unwrap_or(0).to_string();
                                        let l = egui::TextEdit::singleline(&mut w);
                                        if ui.add_sized(LABEL_SIZE, l).changed() {
                                            if let Ok(r) = w.parse::<i32>() {
                                                part.hp_total = Some(r);
                                                if let Err(e) = dbs.create_update_part(
                                                    part.to_owned(),
                                                    key.to_owned(),
                                                ) {
                                                    println!("Key: {:?}", key);
                                                    println!("Update error: {:?}", e);
                                                }
                                            }
                                        };
                                        Ok::<(), String>(())
                                    });
                                    ui.horizontal(|ui| {
                                        separator(ui);
                                        let l = egui::SelectableLabel::new(false, "HP current");
                                        let _ = ui.add_sized(LABEL_SIZE, l).clicked();

                                        let mut w = part.hp_current.unwrap_or(0).to_string();
                                        let l = egui::TextEdit::singleline(&mut w);
                                        if ui.add_sized(LABEL_SIZE, l).changed() {
                                            if let Ok(r) = w.parse::<i32>() {
                                                part.hp_current = Some(r);
                                                if let Err(e) = dbs.create_update_part(
                                                    part.to_owned(),
                                                    key.to_owned(),
                                                ) {
                                                    println!("Key: {:?}", key);
                                                    println!("Update error: {:?}", e);
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
                        egui::ScrollArea::vertical().show(ui, |ui| {
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
                                        key.to_owned(),
                                    ) {
                                        println!("Key: {:?}", key);
                                        println!("Update error: {:?}", e);
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
                            for (k, v) in part
                                .attributes
                                .iter_mut()
                                .filter(|(k, _)| k.key() != "Blurb")
                            {
                                ui.horizontal(|ui| {
                                    // Deal with the label.
                                    let label = k.key().split("_").last().unwrap_or_default();
                                    let l = egui::SelectableLabel::new(false, label);
                                    let _ = ui.add_sized(LABEL_SIZE, l).clicked();
                                    // Deal with the numerical value.
                                    let mut w = v.value_num().unwrap_or(0).to_string();
                                    let l = egui::TextEdit::singleline(&mut w);
                                    if ui.add_sized(SMALL_LABEL_SIZE, l).changed() {
                                        if let Ok(r) = w.parse::<i64>() {
                                            v.update_value_num_by_ref(Some(r));
                                            if let Err(e) = dbs.create_update_attribute(
                                                k.to_owned(),
                                                v.to_owned(),
                                                key.to_owned(),
                                            ) {
                                                println!("Key: {:?}", key);
                                                println!("Update error: {:?}", e);
                                            }
                                        }
                                    };
                                    // Deal with the string value.
                                    let default = String::new();
                                    let mut w =
                                        v.value_text().as_ref().unwrap_or(&default).to_string();
                                    let l = egui::TextEdit::singleline(&mut w);
                                    if ui.add_sized(LABEL_SIZE, l).changed() {
                                        v.update_value_text_by_ref(Some(w));
                                        if let Err(e) = dbs.create_update_attribute(
                                            k.to_owned(),
                                            v.to_owned(),
                                            key.to_owned(),
                                        ) {
                                            println!("Key: {:?}", key);
                                            println!("Update error: {:?}", e);
                                        }
                                    };
                                    Ok::<(), String>(())
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

fn filter_parts<'a>(dbs: &'a LoadedDbs, part: &InputCharacter) -> Vec<&'a str> {
    dbs.permitted_parts
        .iter()
        .filter(|p| p.part_type == part.part_type)
        .map(|p| p.part_name.as_ref())
        .collect::<Vec<_>>()
}
