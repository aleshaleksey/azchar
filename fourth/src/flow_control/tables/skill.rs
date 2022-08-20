use crate::flow_control::*;
use crate::AZCharFourth;

use eframe;
// use eframe::egui::Widget;
use fnv::FnvHashMap;

impl AZCharFourth {
    pub(crate) fn set_skill_tables(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        let char = self
            .current
            .as_mut()
            .expect("Do not call when no character.");

        if ui.selectable_label(false, "Skills Data").clicked() {
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
                        Self::update_skill_table(
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
                        Self::update_skill_table(
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
}
