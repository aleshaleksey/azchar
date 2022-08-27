use self::tables::{AttrOption, DynamicTable, NoteOption, PartOption, Row};
use super::styles;

use azchar_database::character::attribute::AttributeKey;
use azchar_database::character::character::CompleteCharacter;
use azchar_database::root_db::system_config::SystemConfig;
use azchar_database::{CharacterDbRef, LoadedDbs};

use eframe;
use egui::containers::Frame;
use fnv::FnvHashMap;
use std::path::PathBuf;

const LEVEL: &str = "Level";
const PROFICIENCY: &str = "Proficiency";
const STRENGTH: &str = "Strength";
const REFLEX: &str = "Reflex";
const TOUGHNESS: &str = "Toughness";
const ENDURANCE: &str = "Endurance";
const INTELLIGENCE: &str = "Intelligence";
const JUDGEMENT: &str = "Judgement";
const CHARM: &str = "Charm";
const WILL: &str = "Will";

const MAIN_W: f32 = 460.;

pub(self) fn separator(ui: &mut egui::Ui) {
    ui.add(egui::Separator::default().spacing(3.));
}

pub(crate) struct AZCharFourth {
    frame: Frame,
    db_path: String,
    cfg_path: String,
    dbs: Option<LoadedDbs>,
    char_list: Vec<CharacterDbRef>,
    char_for_deletion: Option<(String, String)>,
    new_char: Option<String>,
    hidden_char_list: bool,
    current: Option<CompleteCharacter>,
    images: FnvHashMap<Option<i64>, egui_extras::RetainedImage>,
    default_img: egui_extras::RetainedImage,
    hidden_main_tables: bool,
    main_attr_table: [Row; 6],
    main_level_pro_table: [Row; 2],
    main_stat_table: [Row; 8],
    hidden_skill_tables: bool,
    d20_skill_table: Box<DynamicTable>,
    d100_skill_table: Box<DynamicTable>,
    hidden_resource_tables: bool,
    resources_basic: Box<DynamicTable>,
    resources_points: Box<DynamicTable>,
    resources_body_hp: Box<DynamicTable>,
    hidden_notes: bool,
    note_window: NoteOption,
    hidden_attacks: bool,
    hidden_specials: bool,
    hidden_inventory: bool,
    hidden_spells: bool,
    part_window: PartOption,
    attr_option: AttrOption,
}

impl AZCharFourth {
    pub(crate) fn with_frame(f: Frame) -> Self {
        let default_img =
            egui_extras::RetainedImage::from_image_bytes("-9999", include_bytes!("default.jpg"))
                .unwrap();
        Self {
            frame: f,
            db_path: String::from("fusion.db"),
            cfg_path: String::from("examples/cjfusion.toml"),
            dbs: None,
            char_list: Vec::new(),
            char_for_deletion: None,
            new_char: None,
            hidden_char_list: true,
            hidden_main_tables: false,
            hidden_skill_tables: false,
            hidden_resource_tables: false,
            hidden_attacks: true,
            hidden_specials: true,
            hidden_inventory: true,
            hidden_spells: true,
            hidden_notes: true,
            note_window: NoteOption::None,
            part_window: PartOption::None,
            attr_option: AttrOption::None,
            current: None,
            images: FnvHashMap::default(),
            default_img,
            main_level_pro_table: [Row::default(), Row::default()],
            main_attr_table: [
                Row::default(),
                Row::default(),
                Row::default(),
                Row::default(),
                Row::default(),
                Row::default(),
            ],
            main_stat_table: [
                Row::default(),
                Row::default(),
                Row::default(),
                Row::default(),
                Row::default(),
                Row::default(),
                Row::default(),
                Row::default(),
            ],
            d100_skill_table: Box::new(DynamicTable::default()),
            d20_skill_table: Box::new(DynamicTable::default()),
            resources_basic: Box::new(DynamicTable::default()),
            resources_points: Box::new(DynamicTable::default()),
            resources_body_hp: Box::new(DynamicTable::default()),
        }
    }
}

fn get_sys_config(cfg_path: &str, db_path: &str) -> Result<(), String> {
    let cfg = SystemConfig::from_config(&PathBuf::from(cfg_path))?;
    cfg.into_system_single(&PathBuf::from(db_path))?;
    Ok(())
}

fn create_new_char(
    new_name: &str,
    dbs: &mut Option<LoadedDbs>,
) -> Result<Vec<CharacterDbRef>, String> {
    if let Some(ref mut dbs) = dbs {
        dbs.create_sheet(new_name)?;
        Ok(dbs.list_characters()?)
    } else {
        Err(String::from("No DB loaded."))
    }
}

impl eframe::App for AZCharFourth {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let style = styles::style();
        egui::CentralPanel::default()
            .frame(self.frame.to_owned())
            .show(ctx, |ui| {
                ui.set_style(style);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // First here we insert the line for potential system reference.
                        ui.label("System reference document:");
                        ui.text_edit_singleline(&mut self.cfg_path);

                        if ui.button("Create System").clicked() {
                            match get_sys_config(&self.cfg_path, &self.db_path) {
                                Ok(_) => {}
                                Err(e) => println!("Couldn't create system: {:?}", e),
                            };
                        }
                    });
                    ui.horizontal(|ui| {
                        // Then we insert the line for potential DB path.
                        ui.label("Core DB path:");
                        ui.text_edit_singleline(&mut self.db_path);
                        if ui.button("Load System").clicked() {
                            match self.load_system() {
                                Err(e) => println!("Could not connect: {:?}", e),
                                Ok(_) => self.hidden_char_list = false,
                            };
                        };
                    });
                    // Set a list of characters.
                    // We cannot iter because of burrow.)
                    // NB: We can hide the list.
                    // A button to hide the list of characters.
                    if !self.char_list.is_empty() {
                        let label = if !self.hidden_char_list {
                            "Hide List"
                        } else {
                            "Show List"
                        };
                        if ui.button(label).clicked() {
                            self.hidden_char_list = !self.hidden_char_list;
                        }
                    }
                    if !self.hidden_char_list {
                        ui.heading("Character List:");
                        for i in 0..self.char_list.len() {
                            // The button for each independant character.
                            ui.horizontal(|ui| {
                                let c = &self.char_list[i];
                                let c_name = c.name().to_owned();
                                let c_uuid = c.uuid().to_owned();
                                ui.label(format!("{}.) ", i));
                                if ui.button(format!("{} ({})", c_name, c_uuid)).clicked() {
                                    match self.load_character(&c_name, &c_uuid) {
                                        Ok(_) => self.hidden_char_list = true,
                                        Err(err) => println!("Could not load character: {}", err),
                                    };
                                };
                                if ui.button("Delete").clicked() {
                                    self.char_for_deletion =
                                        Some((c_name.to_owned(), c_uuid.to_owned()));
                                }
                                // TODO: Do this properly.
                                if ui.button("Export (JSON)").clicked() {
                                    let dbs = self.dbs.as_mut().expect("DBS loaded");
                                    if let Ok(char) = dbs.load_character((c_name, c_uuid)) {
                                        let name = format!("{}-{}.json", char.name(), char.uuid());
                                        let file = match std::fs::File::create(name) {
                                            Ok(f) => f,
                                            Err(e) => {
                                                println!("Error: {:?}", e);
                                                return;
                                            }
                                        };
                                        if let Err(e) = serde_json::to_writer_pretty(file, &char) {
                                            println!("Couldn't export character: {:?}.", e);
                                        };
                                    }
                                }
                            });
                        }
                        self.delete_dialog(ctx);
                        // Create new character.
                        ui.horizontal(|ui| {
                            match (
                                &mut self.new_char,
                                ui.button("Create New Character.").clicked(),
                            ) {
                                (&mut Some(ref new_name), true) => {
                                    match create_new_char(new_name, &mut self.dbs) {
                                        Ok(chars) => self.char_list = chars,
                                        Err(e) => println!("Erro creating character: {:?}", e),
                                    };
                                    self.new_char = None;
                                }
                                (Some(ref mut new_name), false) => {
                                    ui.text_edit_singleline(new_name);
                                }
                                (ref mut c, true) => {
                                    **c = Some(String::from("Lord Stupid IV"));
                                }
                                _ => {}
                            }
                        });
                    }

                    // Display the character.
                    if let Some(ref mut char) = self.current {
                        separator(ui);
                        ui.heading(char.name());
                        separator(ui);
                        self.set_main_tables(ui, ctx);
                        separator(ui);
                        self.set_resource_tables(ui, ctx);
                        separator(ui);
                        self.set_skill_tables(ui, ctx);
                        separator(ui);
                        self.set_parts(ui, ctx);
                        separator(ui);
                        self.set_notes(ui, ctx);
                        separator(ui);
                    }
                });
            });
    }
}

impl AZCharFourth {
    fn delete_dialog(&mut self, ctx: &egui::Context) {
        let cd = self.char_for_deletion.clone();
        let char_for_deletion = &mut self.char_for_deletion;
        let char_list = &mut self.char_list;
        let dbs = &mut self.dbs;
        let current_key = self
            .current
            .as_ref()
            .map(|c| (c.name().to_owned(), c.uuid().to_owned()));
        let current = &mut self.current;

        if let (Some((n, u)), Some(ref mut dbs)) = (cd, dbs) {
            egui::Area::new("part-details")
                .default_pos(egui::pos2(32.0, 32.0))
                .show(ctx, |ui| {
                    ui.set_style(styles::style());
                    self.frame.show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.label(
                                "Confirm character deletion.\nA deleted character is gone forever.",
                            );
                            ui.horizontal(|ui| {
                                if ui.button("Delete!").clicked() {
                                    match dbs.delete_character(n.to_owned(), u.to_owned()) {
                                        Err(e) => println!("Failed to delete: {:?}", e),
                                        Ok(_) => match dbs.list_characters() {
                                            Err(e) => println!("Failed to delete: {:?}", e),
                                            Ok(l) => *char_list = l,
                                        },
                                    };
                                    if let Some((rn, ru)) = current_key {
                                        if rn == n && ru == u {
                                            *current = None;
                                        }
                                    }
                                    *char_for_deletion = None;
                                };
                                if ui.button("No!").clicked() {
                                    *char_for_deletion = None;
                                };
                            });
                        });
                    });
                });
        }
    }
}

mod connection;
mod images;
mod tables;
