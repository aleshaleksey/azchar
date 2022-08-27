use crate::flow_control::connection::CharIdPack;
use crate::flow_control::*;
use crate::AZCharFourth;
// use eframe::egui::Widget;

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
            separator(ui);
            ui.horizontal(|ui| {
                let char_id = char.id().unwrap_or_default();
                let proficiency = char
                    .attribute_map
                    .as_mut()
                    .expect("Always set.")
                    .get(&AttributeKey::new(PROFICIENCY.to_string(), char_id))
                    .map(|v| v.value_num())
                    .unwrap_or_default();
                match self
                    .d20_skill_table
                    .d20_skill_table(proficiency, ui, MAIN_W / 2.)
                {
                    Err(e) => println!("Error d20-skill table: {:?}", e),
                    Ok(dat) if !dat.is_empty() => {
                        Self::update_skill_table(
                            CharIdPack::from_complete(char),
                            dat,
                            &mut self.dbs,
                            "d20",
                            char.attribute_map.as_mut().expect("Always set."),
                            &mut self.d20_skill_table,
                        );
                    }
                    _ => {}
                }
                separator(ui);
                match self.d100_skill_table.d100_skill_table(ui, MAIN_W / 2.) {
                    Err(e) => println!("Error d100-skill table: {:?}", e),
                    Ok(dat) if !dat.is_empty() => {
                        Self::update_skill_table(
                            CharIdPack::from_complete(char),
                            dat,
                            &mut self.dbs,
                            "d100",
                            char.attribute_map.as_mut().expect("Always set."),
                            &mut self.d100_skill_table,
                        );
                    }
                    _ => {}
                }
            });
        }
    }
}
