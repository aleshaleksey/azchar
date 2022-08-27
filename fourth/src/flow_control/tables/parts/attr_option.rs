use super::part_option::PartKeys;
use super::*;
use crate::flow_control::*;

use azchar_database::character::attribute::InputAttribute;
use azchar_database::character::character::CompleteCharacter;

#[derive(Clone, Debug)]
pub(crate) enum AttrOption {
    /// This is the idx of the part on the parts vector.
    New(InputAttribute),
    None,
}

impl AttrOption {
    pub(crate) fn set_add_attribute_dialog(
        &mut self,
        ui: &mut egui::Ui,
        current: &mut CompleteCharacter,
        dbs: &mut LoadedDbs,
        p_keys: &PartKeys,
    ) -> Result<(), String> {
        if let AttrOption::New(ref mut new_attr) = self {
            let displayed_key = new_attr
                .key
                .split('_')
                .last()
                .unwrap_or_default()
                .to_owned();

            ui.horizontal(|ui| {
                let part = &mut current.parts[p_keys.idx];
                egui::ComboBox::from_label("")
                    .width(LABEL_SIZE[0])
                    .selected_text(displayed_key)
                    .show_ui(ui, |ui| {
                        let part_attrs = &part.attributes;
                        let permitted = dbs.permitted_attrs.iter().filter(|pa| {
                            part_attrs.iter().all(|x| x.0.key() != &pa.key)
                                && pa
                                    .part_name
                                    .as_ref()
                                    .map(|x| x == &p_keys.character_type)
                                    .unwrap_or(true)
                                && pa
                                    .part_type
                                    .as_ref()
                                    .map(|x| *x == p_keys.part_type)
                                    .unwrap_or(true)
                        });
                        for v in permitted {
                            let displayed_tail =
                                v.key.split('_').last().unwrap_or_default().to_owned();
                            ui.selectable_value(
                                &mut new_attr.key,
                                v.key.to_owned(),
                                displayed_tail,
                            );
                        }
                    });

                if let Some(p) = dbs.permitted_attrs.iter().find(|pa| {
                    (&pa.key == &new_attr.key)
                        && pa
                            .part_name
                            .as_ref()
                            .map(|x| x == &p_keys.character_type)
                            .unwrap_or(true)
                        && pa
                            .part_type
                            .as_ref()
                            .map(|x| *x == p_keys.part_type)
                            .unwrap_or(true)
                }) {
                    new_attr.description = Some(p.attribute_description.to_owned());
                    let l = egui::SelectableLabel::new(false, &p.attribute_description);
                    let _ = ui.add_sized(LABEL_SIZE, l).clicked();
                }
                if ui.button("Add Attribute").clicked() {
                    let char_key = (current.name().to_owned(), current.uuid().to_owned());
                    match dbs.create_attribute(new_attr.to_owned(), char_key) {
                        Err(e) => return Err(format!("Couldn't add attribute: {:?}", e)),
                        Ok(mut c) => {
                            c.create_attribute_map();
                            *current = c;
                        }
                    }
                }
                Ok(())
            }).inner?;
        }
        Ok(())
    }
}
