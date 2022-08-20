use crate::flow_control::*;
use crate::AZCharFourth;

use eframe;
// use eframe::egui::Widget;
use fnv::FnvHashMap;

impl AZCharFourth {
    pub(crate) fn set_resource_tables(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        let char = self
            .current
            .as_mut()
            .expect("Do not call when no character.");

        if ui.selectable_label(false, "Basic Character Data").clicked() {
            self.hidden_skill_tables = !self.hidden_skill_tables;
        }
        if !self.hidden_skill_tables {
            ui.horizontal(|ui| {
                let proficiency = self
                    .current_attributes
                    .get(&AttributeKey::new(
                        PROFICIENCY.to_string(),
                        char.id().unwrap_or_default(),
                    ))
                    .map(|v| v.value_num())
                    .unwrap_or_default();
                match self
                    .d20_skill_table
                    .d20_skill_table(proficiency, ui, MAIN_W / 2.)
                {
                    Err(e) => println!("Error d20-skill table: {:?}", e),
                    Ok(dat) if !dat.is_empty() => {
                        Self::update_resource_table(
                            char,
                            dat,
                            &mut self.dbs,
                            "d20",
                            &mut self.current_attributes,
                            &mut self.d20_skill_table,
                        );
                    }
                    _ => {}
                }
                match self.d100_skill_table.d100_skill_table(ui, MAIN_W / 2.) {
                    Err(e) => println!("Error d100-skill table: {:?}", e),
                    Ok(dat) if !dat.is_empty() => {
                        Self::update_resource_table(
                            char,
                            dat,
                            &mut self.dbs,
                            "d100",
                            &mut self.current_attributes,
                            &mut self.d20_skill_table,
                        );
                    }
                    _ => {}
                }
            });
        }
    }

    fn update_resource_table(
        char: &CompleteCharacter,
        // (row, column)
        references: Vec<(usize, usize)>,
        dbs: &mut Option<LoadedDbs>,
        skill_kind: &str,
        attributes: &mut FnvHashMap<AttributeKey, AttributeValue>,
        table: &mut Box<DynamicTable>,
    ) {
        let dbs = match dbs.as_mut() {
            Some(d) => d,
            None => return,
        };
        for (r_idx, c_idx) in references {
            let attr_label = &table.column_labels[c_idx].key;
            let skill_label = &table.row_labels[c_idx].key;

            let key = format!("{}_skill_{}_{}", skill_kind, skill_label, attr_label);
            let of = char.id().expect("This character has been through the DB.");
            // If we have a valid value in this cell, we work, if not we skip.
            let val_n = match table.cells[r_idx][c_idx].parse() {
                Ok(v) => Some(v),
                Err(_) => continue,
            };
            let key = AttributeKey::new(key, of);

            if let Some(val) = attributes.get_mut(&key) {
                val.update_value_num_by_ref(val_n);
                println!("Val updated to: {:?}", val);
                let identifier = (char.name().to_owned(), char.uuid().to_owned());
                match dbs.create_update_attribute(key, val.to_owned(), identifier) {
                    Err(e) => println!("Couldn't update attribute: {:?}", e),
                    _ => {}
                }
            }
        }
    }
}
