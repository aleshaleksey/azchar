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

        if ui.selectable_label(false, "Character Resource").clicked() {
            self.hidden_resource_tables = !self.hidden_resource_tables;
        }
        if !self.hidden_resource_tables {
            ui.horizontal(|ui| {
                match self
                    .resources_basic
                    .set_attr_based_resource("BASICS", ui, MAIN_W / 3.)
                {
                    Err(e) => println!("Error updating basics: {:?}", e),
                    Ok(dat) if !dat.is_empty() => {
                        Self::update_text_attr_table(
                            char,
                            dat,
                            &mut self.dbs,
                            &mut self.current_attributes,
                            &mut self.resources_basic,
                        );
                    }
                    _ => {}
                }
                match self
                    .resources_points
                    .set_attr_based_resource("BASICS", ui, MAIN_W / 3.)
                {
                    Err(e) => println!("Error updating basics: {:?}", e),
                    Ok(dat) if !dat.is_empty() => {
                        Self::update_text_attr_table(
                            char,
                            dat,
                            &mut self.dbs,
                            &mut self.current_attributes,
                            &mut self.resources_points,
                        );
                    }
                    _ => {}
                }
                // match self
                //     .d20_skill_table
                //     .d20_skill_table(proficiency, ui, MAIN_W / 2.)
                // {
                //     Err(e) => println!("Error d20-skill table: {:?}", e),
                //     Ok(dat) if !dat.is_empty() => {
                //         Self::update_resource_table(
                //             char,
                //             dat,
                //             &mut self.dbs,
                //             "d20",
                //             &mut self.current_attributes,
                //             &mut self.d20_skill_table,
                //         );
                //     }
                //     _ => {}
                // }
                // match self.d100_skill_table.d100_skill_table(ui, MAIN_W / 2.) {
                //     Err(e) => println!("Error d100-skill table: {:?}", e),
                //     Ok(dat) if !dat.is_empty() => {
                //         Self::update_resource_table(
                //             char,
                //             dat,
                //             &mut self.dbs,
                //             "d100",
                //             &mut self.current_attributes,
                //             &mut self.d20_skill_table,
                //         );
                //     }
                //     _ => {}
                // }
            });
        }
    }
}
