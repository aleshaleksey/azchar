use crate::flow_control::*;
use crate::AZCharFourth;

use azchar_database::character::*;

// use eframe::egui::Widget;
#[derive(Clone, Debug)]
pub(crate) enum NoteOption {
    Existing(i64),
    New(InputNote),
    None,
}

impl NoteOption {
    pub(crate) fn is_none(&self) -> bool {
        matches!(self, &NoteOption::None)
    }
    pub(crate) fn is_existing(&self) -> bool {
        matches!(self, &NoteOption::Existing(_))
    }
    pub(crate) fn is_this_note(&self, id: i64) -> bool {
        match self {
            NoteOption::Existing(i) => id == *i,
            _ => false,
        }
    }
    fn set_and_update_new_note(
        &mut self,
        char: &mut CompleteCharacter,
        dbs: &mut LoadedDbs,
        ui: &mut egui::Ui,
    ) -> Result<(), String> {
        let exit = match self {
            NoteOption::New(new) => {
                if new.content.is_none() {
                    new.content = Some(String::with_capacity(100));
                }
                let finish = ui
                    .horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut new.title));
                        if ui.button("Save New Note").clicked() {
                            match dbs.add_note(
                                char.name().to_owned(),
                                char.uuid().to_owned(),
                                new.to_owned(),
                            ) {
                                Err(e) => println!("Error updating note: {:?}", e),
                                Ok(n) => {
                                    let new = NoteOption::Existing(n.id);
                                    char.notes.push(n);
                                    return Ok(new);
                                }
                            }
                        };
                        Ok::<_, String>(NoteOption::None)
                    })
                    .inner?;
                if finish.is_existing() {
                    let _ = egui::TextEdit::multiline(new.content.as_mut().unwrap())
                        .frame(true)
                        .margin(egui::Vec2::new(2., 2.))
                        .desired_rows(20)
                        .lock_focus(true)
                        .cursor_at_end(true);
                }
                finish
            }
            _ => NoteOption::None,
        };
        if exit.is_existing() {
            *self = exit;
        }
        Ok(())
    }
}

impl AZCharFourth {
    pub(crate) fn set_notes(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        let char = self
            .current
            .as_mut()
            .expect("Do not call when no character.");
        let dbs = self
            .dbs
            .as_mut()
            .expect("We have a DB because we have a character.");

        if ui.selectable_label(false, "Character Notes").clicked() {
            self.hidden_notes = !self.hidden_notes;
        }
        if !self.hidden_notes {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let new_clicked = ui.selectable_label(false, "Add New Note").clicked();
                if self.note_window.is_none() && new_clicked {
                    self.note_window = NoteOption::New(InputNote::default());
                }
                if let Err(e) = self.note_window.set_and_update_new_note(char, dbs, ui) {
                    println!("Error with new note: {:?}", e);
                }
                for i in (0..char.notes.len()).rev() {
                    let row = ui.horizontal(|ui| {
                        let n = &char.notes[i];
                        // Label
                        let id = n.id.to_string();
                        let id = egui::SelectableLabel::new(false, &id);
                        let id = ui.add_sized([30., 21.], id).clicked();
                        // Title.
                        let lab = egui::SelectableLabel::new(false, &n.title);
                        let lab = ui.add_sized([30., 21.], lab).clicked();
                        // Date.
                        let date = egui::SelectableLabel::new(false, &n.date);
                        let date = ui.add_sized([30., 21.], date).clicked();
                        if self.note_window.is_none() && (lab || id || date) {
                            self.note_window = NoteOption::Existing(n.id);
                            println!("Current status: {:?}", self.note_window);
                        } else if (lab || id || date) && self.note_window.is_existing() {
                            self.note_window = NoteOption::None;
                            println!("Current status: {:?}", self.note_window);
                        }
                    }).response.rect;

                    // If titles are set, and we hav selected this note.
                    if self.note_window.is_this_note(char.notes[i].id) {
                        let name = char.name().to_owned();
                        let uuid = char.uuid().to_owned();

                        let n = &mut char.notes[i];
                        if n.content.is_none() {
                            n.content = Some(String::with_capacity(100));
                        }
                        if let Some(ref mut content) = n.content {
                            let width = row.max.x - row.min.x;
                            let note_edit = egui::TextEdit::multiline(content)
                                .frame(true)
                                .margin(egui::Vec2::new(2., 2.))
                                .desired_rows(14)
                                .desired_width(width)
                                .lock_focus(true)
                                .cursor_at_end(true);
                            if ui.add(note_edit).changed() {
                                if let Err(e) =
                                    dbs.update_note(name.to_owned(), uuid.to_owned(), n.to_owned())
                                {
                                    println!("Error updating note: {:?}", e);
                                }
                            }
                        }
                    }
                }
            });
        }
    }
}
