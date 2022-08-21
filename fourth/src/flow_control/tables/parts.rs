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
    pub(crate) fn is_this_part(&self, id: i64) -> bool {
        match self {
            Self::Existing(i) => id == *i,
            _ => false,
        }
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
                                    let new = PartOption::None;
                                    *char = updated_char;
                                    char.create_attribute_map();
                                    println!("{:?}", char);
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
        let char = self
            .current
            .as_mut()
            .expect("Do not call when no character.");
        let dbs = self
            .dbs
            .as_mut()
            .expect("We have a DB because we have a character.");

        if ui.selectable_label(false, "Character Attacks").clicked() {
            self.hidden_attacks = !self.hidden_attacks;
        }
        if !self.hidden_attacks {
            ui.separator();
            if let Err(e) = self.set_parts_list(ui, Part::Ability, Some("attack")) {
                println!("We have a problem: {:?}", e);
            };
        }
        ui.separator();

        if ui.selectable_label(false, "Character Specials").clicked() {
            self.hidden_specials = !self.hidden_specials;
        }
        if !self.hidden_specials {
            ui.separator();
            if let Err(e) = self.set_parts_list(ui, Part::Ability, Some("specials")) {
                println!("We have a problem: {:?}", e);
            };
        }
        ui.separator();

        if ui.selectable_label(false, "Character Inventory").clicked() {
            self.hidden_inventory = !self.hidden_inventory;
        }
        if !self.hidden_inventory {
            ui.separator();
            if let Err(e) = self.set_parts_list(ui, Part::InventoryItem, None) {
                println!("We have a problem: {:?}", e);
            };
        }
        ui.separator();

        if ui.selectable_label(false, "Character Spells").clicked() {
            self.hidden_spells = !self.hidden_spells;
        }
        if !self.hidden_spells {
            ui.separator();
            if let Err(e) = self.set_parts_list(ui, Part::Ability, Some("spells")) {
                println!("We have a problem: {:?}", e);
            };
        }
        if let PartOption::Existing(id) = self.part_window {
            self.display_part_details(ui, id, ctx);
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

    fn display_part_details(&mut self, ui: &mut egui::Ui, part_id: i64, ctx: &egui::Context) {
        egui::Area::new("part-details")
            .default_pos(egui::pos2(32.0, 32.0))
            .show(ctx, |ui| {
                crate::styles::default_frame().show(ui, |ui| {
                    ui.selectable_label(false, "Floating text!");
                    ui.selectable_label(false, "Doubling text!");
                    ui.selectable_label(false, "Integer text!");
                });
            });
    }
}

fn filter_parts<'a>(dbs: &'a LoadedDbs, part: &InputCharacter) -> Vec<&'a str> {
    dbs.permitted_parts
        .iter()
        .filter(|p| p.part_type == part.part_type)
        .map(|p| p.part_name.as_ref())
        .collect::<Vec<_>>()
}
