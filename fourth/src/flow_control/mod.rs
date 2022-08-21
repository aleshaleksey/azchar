use self::tables::{DynamicTable, NoteOption, PartOption, Row};
use super::styles;

use azchar_database::character::attribute::AttributeKey;
use azchar_database::character::character::CompleteCharacter;
use azchar_database::root_db::system_config::SystemConfig;
use azchar_database::{CharacterDbRef, LoadedDbs};

use eframe;
use eframe::egui::Widget;
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

pub(crate) struct AZCharFourth {
    frame: Frame,
    db_path: String,
    cfg_path: String,
    dbs: Option<LoadedDbs>,
    char_list: Vec<CharacterDbRef>,
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
                                    println!("We shall pretend to delete {}", c_name);
                                }
                            });
                        }
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
                        ui.separator();
                        ui.heading(char.name());
                        ui.separator();
                        self.set_main_tables(ui, ctx);
                        ui.separator();
                        self.set_resource_tables(ui, ctx);
                        ui.separator();
                        self.set_skill_tables(ui, ctx);
                        ui.separator();
                        self.set_parts(ui, ctx);
                        ui.separator();
                        self.set_notes(ui, ctx);
                        ui.separator();
                    }
                });
            });
    }
}

mod connection;
mod tables;
