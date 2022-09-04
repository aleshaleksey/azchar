use crate::file_dialog::{FileFilters, FileManager, FileSelection};
use crate::main_dialog::{self, export, import, AZCharFourth};
use crate::separator;
use crate::styles;

use azchar_database::{CharacterDbRef, LoadedDbs};

use eframe::App;
use egui::{Frame, Style};
use fnv::FnvHashMap;

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};

/// This represents a file manager.
/// It shows a list of files (so it must store them).
/// It allows the selection of a file so that must be returned.
/// It should ideally also show an image thumbnail...
pub(crate) struct FlowController {
    pub(crate) main_dialog: main_dialog::AZCharFourth,
    pub(crate) file_dialog: FileManager,
    pub(crate) state: FlowControllerState,
}

impl FlowController {
    pub(crate) fn with_frame(f: Frame) -> Self {
        Self {
            main_dialog: main_dialog::AZCharFourth::with_frame(f),
            file_dialog: FileManager::new(),
            state: FlowControllerState::Main,
        }
    }
}

#[derive(Clone, Debug)]
pub enum For {
    ImportCharacter,
    ExportCharacter(CharacterDbRef),
    ImportPart,
    ExportPart(usize),
    ImportImage(i64),
    None,
}

#[derive(Clone, Debug)]
pub(crate) enum FlowControllerState {
    Main,
    FileDialog(For),
}

impl App for FlowController {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        match self.state {
            FlowControllerState::Main => {
                self.main_dialog.update(ctx, frame);
                if !matches!(self.main_dialog.file_dialog, For::None) {
                    self.state =
                        FlowControllerState::FileDialog(self.main_dialog.file_dialog.clone());
                }
                match self.state {
                    FlowControllerState::FileDialog(For::ImportCharacter)
                    | FlowControllerState::FileDialog(For::ImportPart) => {
                        let filters =
                            FileFilters::files(vec!["json".to_string(), "JSON".to_string()]);
                        self.file_dialog.set_filters(filters);
                    }
                    FlowControllerState::FileDialog(For::ImportImage(_)) => {
                        let v = vec!["png", "jpg", "jpeg", "bmp", "gif"]
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>();
                        let filters = FileFilters::files(v);
                        self.file_dialog.set_filters(filters);
                    }
                    FlowControllerState::FileDialog(_) => {
                        self.file_dialog.set_filters(FileFilters::dirs())
                    }
                    _ => {}
                }
            }
            FlowControllerState::FileDialog(ref funct) => {
                self.file_dialog.update(ctx, frame);
                let mut res = Ok(());
                match self.file_dialog.selection {
                    FileSelection::Undecided | FileSelection::Cancelled => {}
                    FileSelection::Selected(ref path) => {
                        println!("Path:{:?}, for:{:?}", path, funct);
                        match (
                            funct,
                            &mut self.main_dialog.dbs,
                            &mut self.main_dialog.current,
                        ) {
                            (For::ImportCharacter, Some(ref mut dbs), _) => {
                                import::character(dbs, path.to_owned());
                                match dbs.list_characters() {
                                    Ok(l) => self.main_dialog.char_list = l,
                                    Err(e) => res = Err(e),
                                };
                            }
                            (For::ExportCharacter(c), Some(ref mut dbs), _) => {
                                let c_name = c.name().to_owned();
                                let c_uuid = c.uuid().to_owned();
                                res = export::character(dbs, &c_name, &c_uuid, path.to_owned());
                            }
                            (For::ImportPart, Some(ref mut dbs), Some(ref mut c)) => {
                                import::part(dbs, c, path.to_owned());
                                let c_name = c.name().to_owned();
                                let c_uuid = c.uuid().to_owned();
                                match dbs.load_character((c_name, c_uuid)) {
                                    Ok(new) => *c = new,
                                    Err(e) => res = Err(e),
                                };
                            }
                            (For::ExportPart(i), Some(ref mut dbs), Some(ref c)) => {
                                res = export::part(&c.parts[*i], path.to_owned());
                            }
                            (For::ImportImage(i), Some(ref mut dbs), Some(ref mut c)) => {
                            let c_name = c.name().to_owned();
                            let c_uuid = c.uuid().to_owned();
                            let images = &mut self.main_dialog.images;
                            let mut p = c.parts.iter_mut().find(|p| p.id() == Some(*i));
                            let c_image = if let Some(ref mut c) = p {
                                &mut c.image
                            } else {
                                // This can also go very vey wrong...
                                &mut c.image
                            };
                                res = AZCharFourth::set_image(
                                    dbs, c_image, images, c_name, c_uuid, *i, path.to_path_buf(),
                                );
                            }
                            _ => {}
                        }
                    }
                }
                // Importantly we must reset the state.
                if !matches!(self.file_dialog.selection, FileSelection::Undecided) {
                    self.state = FlowControllerState::Main;
                    self.main_dialog.file_dialog = For::None;
                    self.file_dialog.selection = FileSelection::Undecided;
                }
                if let Err(e) = res {
                    main_dialog::error_dialog::fill(e, &mut self.main_dialog.error_dialog);
                }
            }
        }
    }
}
