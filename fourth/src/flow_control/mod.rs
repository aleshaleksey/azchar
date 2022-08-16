use azchar_database::character::character::CompleteCharacter;
use azchar_database::{CharacterDbRef, LoadedDbs};
use azchar_server::{Request, Response};

use eframe::egui::Widget;
use eframe::{self, egui};

use fnv::FnvHashMap;

pub(crate) struct AZCharFourth {
    db_path: String,
    cfg_path: String,
    dbs: Option<LoadedDbs>,
    char_list: Vec<CharacterDbRef>,
    hidden_list: bool,
    current: Option<CompleteCharacter>,
    images: FnvHashMap<i64, egui_extras::RetainedImage>,
    default_img: egui_extras::RetainedImage,
}

impl Default for AZCharFourth {
    fn default() -> Self {
        let default_img = egui_extras::RetainedImage::from_image_bytes(
            "-9999",
            include_bytes!("default.jpg"),
        )
        .unwrap();
        Self {
            db_path: String::from("fusion.db"),
            cfg_path: String::new(),
            dbs: None,
            char_list: Vec::new(),
            hidden_list: true,
            current: None,
            images: FnvHashMap::default(),
            default_img,
        }
    }
}

impl eframe::App for AZCharFourth {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                // First here we insert the line for potential system reference.
                ui.label("System reference document:");
                ui.text_edit_singleline(&mut self.cfg_path);

                if ui.button("Create System").clicked() {
                    println!("Unsupported!");
                }
            });
            ui.horizontal(|ui| {
                // Then we insert the line for potential DB path.
                ui.label("Core DB path:");
                ui.text_edit_singleline(&mut self.db_path);
                if ui.button("Load System").clicked() {
                    match self.load_system() {
                        Err(e) => println!("Could not connect: {:?}", e),
                        Ok(_) => self.hidden_list = false,
                    };
                };
            });

            ui.label(format!("Dbs loaded: {:?}", self.dbs.is_some()));
            ui.label(format!("Character count in DB: {:?}", self.char_list.len()));
            ui.label(format!("Character loaded: {}", self.current.is_some()));
            // Set a list of characters.
            // We cannot iter because of burrow.)
            // NB: We can hide the list.
            if !self.hidden_list {
                ui.heading("Character List:");
                for i in 0..self.char_list.len() {
                    // The button for each independant character.
                    ui.horizontal(|ui| {
                        let c = &self.char_list[i];
                        let c_name = c.name().to_owned();
                        let c_uuid = c.uuid().to_owned();
                        ui.label(format!("{}.) ", i));
                        if ui.button(format!("{} ({})", c_name, c_uuid)).clicked() {
                            println!("We shall pretend to load {}", c_name);
                            if let Err(err) = self.load_character(&c_name, &c_uuid) {
                                println!("Could not load character: {}", err);
                            }
                        };
                        if ui.button("Delete").clicked() {
                            println!("We shall pretend to delete {}", c_name);
                        }
                    });
                }
                // Create new character.
                if ui.button("Create New Character.").clicked() {
                    println!("Pretending to make a new character.");
                }
            }
            // A button to hide the list of characters.
            if !self.char_list.is_empty() {
                let label = if !self.hidden_list {
                    "Hide List"
                } else {
                    "Show List"
                };
                if ui.button(label).clicked() {
                    self.hidden_list = !self.hidden_list;
                }
            }

            // Display the character.
            if let Some(ref char) = self.current {
                ui.heading(char.name());
                // Portrait or default for box.
                let portrait =  self.images.get(&char.id().unwrap()).unwrap_or(&self.default_img);
                {
                    let mut ib = egui::ImageButton::new(portrait.texture_id(ctx), [200., 200.]);
                    if ib.ui(ui).clicked() {
                        println!("We pretend to update the portrait.");
                    }
                }
            }
        });
    }
}

mod connection;
