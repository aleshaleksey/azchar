use crate::flow_control::*;

use azchar_database::character::character::InputCharacter;
use azchar_database::shared::Part;

type Key<'a> = (Part, Option<&'a str>);

fn compare_keys(gui_part: Key, new_type: Part, new_character_type: &str) -> bool {
    gui_part.0 == new_type && gui_part.1.map(|t| t == new_character_type).unwrap_or(true)
}

fn filter_parts<'a>(dbs: &'a LoadedDbs, part: &InputCharacter) -> Vec<&'a str> {
    dbs.permitted_parts
        .iter()
        .filter(|p| p.part_type == part.part_type)
        .map(|p| p.part_name.as_ref())
        .collect::<Vec<_>>()
}

#[derive(Debug, Clone)]
pub(crate) struct PartKeys {
    pub(crate) idx: usize,
    pub(crate) id: i64,
    pub(crate) part_type: Part,
    pub(crate) character_type: String,
}

#[derive(Clone, Debug)]
pub(crate) enum PartOption {
    /// This is the idx of the part on the parts vector.
    ExistingIdx(PartKeys),
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
        matches!(self, &Self::ExistingIdx(_))
    }
    pub(super) fn set_and_update_new_part(
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
                                Err(e) => return Err(format!("{:?}", e)),
                                Ok(updated_char) => {
                                    *char = updated_char;
                                    char.create_attribute_map();
                                    return Ok(true);
                                }
                            }
                        };
                        Ok(false)
                    })
                })
                .inner
                .inner?
            }
            _ => false,
        };
        if exit {
            *self = Self::None;
        }
        Ok(())
    }
}
