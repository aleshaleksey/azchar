use self::table::{DynamicTable, Row};

use azchar_database::character::attribute::{AttributeKey, AttributeValue};
use azchar_database::character::character::CompleteCharacter;
use azchar_database::{CharacterDbRef, LoadedDbs};

use eframe;
use eframe::egui::Widget;

use fnv::FnvHashMap;

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

const MAIN_W: f32 = 320.;

pub(crate) struct AZCharFourth {
    db_path: String,
    cfg_path: String,
    dbs: Option<LoadedDbs>,
    char_list: Vec<CharacterDbRef>,
    hidden_list: bool,
    current: Option<CompleteCharacter>,
    images: FnvHashMap<Option<i64>, egui_extras::RetainedImage>,
    current_attributes: FnvHashMap<AttributeKey, AttributeValue>,
    default_img: egui_extras::RetainedImage,
    main_attr_table: [Row; 6],
    main_level_pro_table: [Row; 2],
    main_stat_table: [Row; 8],
    d20_skill_table: Box<DynamicTable>,
    d100_skill_table: Box<DynamicTable>,
}

impl Default for AZCharFourth {
    fn default() -> Self {
        let default_img =
            egui_extras::RetainedImage::from_image_bytes("-9999", include_bytes!("default.jpg"))
                .unwrap();
        Self {
            db_path: String::from("fusion.db"),
            cfg_path: String::new(),
            dbs: None,
            char_list: Vec::new(),
            hidden_list: true,
            current: None,
            images: FnvHashMap::default(),
            current_attributes: FnvHashMap::default(),
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
        }
    }
}

impl eframe::App for AZCharFourth {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
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

                // Display the character.
                if let Some(ref mut char) = self.current {
                    ui.heading(char.name());
                    self.set_main_tables(ui, ctx);
                    self.set_skill_tables(ui, ctx);
                }
            });
        });
    }
}

impl AZCharFourth {
    fn set_main_tables(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let char = self
            .current
            .as_mut()
            .expect("Do not call when no character.");
        ui.horizontal(|ui| {
            // Portrait or default for box.
            let portrait = self.images.get(&char.id()).unwrap_or(&self.default_img);
            {
                let ib = egui::ImageButton::new(portrait.texture_id(ctx), [128., 128.]);
                if ib.ui(ui).clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("image", &["png", "jpg", "jpeg", "bmp"])
                        .pick_file()
                    {
                        println!("Picked: {:?}", path);
                        let name = char.name().to_owned();
                        let uuid = char.uuid().to_owned();
                        let id = char.id().unwrap();
                        let res = AZCharFourth::set_image(
                            &mut self.dbs,
                            &mut char.image,
                            &mut self.images,
                            name,
                            uuid,
                            id,
                            path,
                        );
                        if let Err(e) = res {
                            println!("Couldn't set image: {:?}", e);
                        }
                    } else {
                        println!("Failed to pick a file.");
                    }
                }
            }
            // Set the three attribute tables.
            ui.vertical(|ui| {
                {
                    let rows = &mut self.main_attr_table;
                    match AZCharFourth::horizontal_table(ui, rows, MAIN_W) {
                        Err(e) => println!("Error: {}", e),
                        Ok(true) => {
                            char.name = rows[0].value.to_owned();
                            if let Ok(n) = rows[1].value.parse() {
                                char.speed = n;
                            }
                            if let Ok(n) = rows[2].value.parse() {
                                char.weight = Some(n);
                            }
                            char.size = Some(rows[3].value.to_owned());
                            if let Ok(n) = rows[4].value.parse() {
                                char.hp_current = Some(n);
                            }
                            if let Ok(n) = rows[5].value.parse() {
                                char.hp_total = Some(n);
                            }
                            let part = char.to_bare_part();

                            let res = AZCharFourth::update_main(&mut self.dbs, part);
                            if let Err(e) = res {
                                println!("Couldn't set image: {:?}", e);
                            }
                        }
                        _ => {}
                    };
                }
                {
                    let rows = &mut self.main_level_pro_table;
                    match AZCharFourth::horizontal_table(ui, rows, MAIN_W) {
                        Err(e) => println!("Error: {}", e),
                        Ok(true) => {
                            let res = AZCharFourth::update_attrs(&mut self.dbs, char, rows);
                            if let Err(e) = res {
                                println!("Couldn't set image: {:?}", e);
                            }
                        }
                        _ => {}
                    };
                }
                {
                    let rows = &mut self.main_stat_table;
                    match AZCharFourth::horizontal_table(ui, rows, MAIN_W) {
                        Err(e) => println!("Error: {}", e),
                        Ok(true) => {
                            let res = AZCharFourth::update_attrs(&mut self.dbs, char, rows);
                            if let Err(e) = res {
                                println!("Couldn't set image: {:?}", e);
                            }
                        }
                        _ => {}
                    };
                }
            });
        });
    }

    fn set_skill_tables(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let char = self
            .current
            .as_mut()
            .expect("Do not call when no character.");
        ui.horizontal(|ui| {
            match self.d20_skill_table.d20_skill_table(ui, MAIN_W / 2.) {
                Err(e) => println!("Error d20-skill table: {:?}", e),
                Ok(ref dat) if !dat.is_empty() => {}
                _ => {}
            }
            match self.d100_skill_table.d100_skill_table(ui, MAIN_W / 2.) {
                Err(e) => println!("Error d20-skill table: {:?}", e),
                Ok(ref dat) if !dat.is_empty() => {}
                _ => {}
            }
        });
    }
}

mod connection;
mod table;
