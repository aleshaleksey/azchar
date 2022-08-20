use super::tables::{DynamicTable, Label, Row};
use super::*;

use azchar_database::character::attribute::{AttributeKey, AttributeValue};
use azchar_database::character::character::CharacterPart;
use azchar_database::character::image as dbimg;
use azchar_database::LoadedDbs;
use azchar_error::ma;

const D20_SKILLS: [&str; 14] = [
    "Awareness",
    "Acting",
    "Agility",
    "Beast Mastery",
    "Convince",
    "Cunning",
    "Faith",
    "Intuition",
    "Knowledge",
    "Scrutiny",
    "Strong Arm",
    "Stealth",
    "Survival",
    "Trickery",
];
const D100_SKILLS: [&str; 13] = [
    "Armourer",
    "Biomedicine",
    "Combat Medicine",
    "Demolition",
    "Engineering",
    "Firearms",
    "Hacking",
    "Melee",
    "Piloting",
    "Research",
    "Surgery",
    "Unarmed",
    "Underworld",
];
const BASICS: [&str; 8] = [
    "Race",
    "Alignment",
    "Height",
    "Hair",
    "Eyes",
    "Age",
    "Skin",
    "Player",
];
const POINTS: [(&str, &str, bool); 8] = [
    ("Flair", "flair", false),
    ("Surge", "surge", false),
    ("Strain", "strain", false),
    ("MP pool", "mp", true),
    ("MP daily", "mp_use_day", true),
    ("Ki pool", "ki", true),
    ("Ki daily", "ki_use_day", true),
    ("Psi daily", "psi_use_day", true),
];
const BODY_PARTS: [&str; 8] = [
    "Head",
    "Neck",
    "Left Arm",
    "Right Arm",
    "Body",
    "Groin",
    "Left Leg",
    "Right Leg",
];
const PROFICIENCY: &str = "proficiency";
pub(super) const PROFICIENCY_CAMEL: &str = "Proficiency";
const BONUS: &str = "bonus";
const BONUS_CAMEL: &str = "Bonus";
const TOTAL_CAMEL: &str = "Total";
const GOV: &str = "governed_by";
const GOV_CAMEL: &str = "Governed By";

use fnv::FnvHashMap;

impl AZCharFourth {
    pub(super) fn load_system(&mut self) -> Result<(), String> {
        let mut dbs = LoadedDbs::custom(&self.db_path)?;
        self.char_list = dbs.list_characters()?;
        self.dbs = Some(dbs);
        Ok(())
    }

    pub(super) fn load_character(&mut self, name: &str, uuid: &str) -> Result<(), String> {
        if let Some(ref mut dbs) = self.dbs {
            let loaded = dbs.load_character((name.to_owned(), uuid.to_owned()))?;
            let mut imagemap = FnvHashMap::default();
            // Insert primary image.
            if let Some(data) = loaded.image().as_ref() {
                let processed = process_image(data)?;
                imagemap.insert(loaded.id(), processed);
            }
            self.current_attributes = loaded
                .attributes()
                .iter()
                .cloned()
                .collect::<FnvHashMap<_, _>>();

            // Insert part images.
            for c in loaded.parts().iter() {
                if let Some(data) = c.image.as_ref() {
                    let processed = process_image(data)?;
                    imagemap.insert(loaded.id(), processed);
                }
                for (k, v) in c.attributes.iter() {
                    self.current_attributes.insert(k.clone(), v.clone());
                }
            }
            self.main_attr_table = [
                Row::new("Name", loaded.name()),
                Row::new("Speed", &loaded.speed.to_string()),
                Row::new(
                    "Weight",
                    &loaded.weight.map(|x| x.to_string()).unwrap_or_default(),
                ),
                Row::new("Size", &loaded.size.to_owned().unwrap_or_default()),
                Row::new(
                    "HP",
                    &loaded.hp_current.map(|x| x.to_string()).unwrap_or_default(),
                ),
                Row::new(
                    "HP total",
                    &loaded.hp_total.map(|x| x.to_string()).unwrap_or_default(),
                ),
            ];

            let main_id = loaded.id().expect("A databse character has an id");
            {
                let level = get_attr_val_num(&self.current_attributes, LEVEL, main_id);
                let proficiency =
                    get_attr_val_num(&self.current_attributes, PROFICIENCY_CAMEL, main_id);
                self.main_level_pro_table = [
                    Row::with_label("Level", &level.to_string(), LEVEL),
                    Row::with_label("Proficiency", &proficiency.to_string(), PROFICIENCY_CAMEL),
                ];
            }
            {
                let str = get_attr_val_num(&self.current_attributes, STRENGTH, main_id);
                let re = get_attr_val_num(&self.current_attributes, REFLEX, main_id);
                let tou = get_attr_val_num(&self.current_attributes, TOUGHNESS, main_id);
                let end = get_attr_val_num(&self.current_attributes, ENDURANCE, main_id);
                let int = get_attr_val_num(&self.current_attributes, INTELLIGENCE, main_id);
                let jud = get_attr_val_num(&self.current_attributes, JUDGEMENT, main_id);
                let cha = get_attr_val_num(&self.current_attributes, CHARM, main_id);
                let wil = get_attr_val_num(&self.current_attributes, WILL, main_id);
                self.main_stat_table = [
                    Row::with_label("STR", &str.to_string(), STRENGTH),
                    Row::with_label("REF", &re.to_string(), REFLEX),
                    Row::with_label("TOU", &tou.to_string(), TOUGHNESS),
                    Row::with_label("END", &end.to_string(), ENDURANCE),
                    Row::with_label("INT", &int.to_string(), INTELLIGENCE),
                    Row::with_label("JUD", &jud.to_string(), JUDGEMENT),
                    Row::with_label("CHA", &cha.to_string(), CHARM),
                    Row::with_label("WIL", &wil.to_string(), WILL),
                ];
            }
            {
                let mut d100_table = DynamicTable::default();
                let column_labels = vec![
                    Label::new(PROFICIENCY_CAMEL, PROFICIENCY),
                    Label::new(BONUS_CAMEL, BONUS),
                    Label::new(TOTAL_CAMEL, TOTAL_CAMEL),
                ];
                d100_table.add_column_labels(column_labels);
                for skill in D100_SKILLS.iter() {
                    let label = Label::new(skill, skill);

                    let key = format!("d100_skill_{}_proficiency", skill);
                    let proficiency = get_attr_val_num_o(&self.current_attributes, key, main_id);
                    let key = format!("d100_skill_{}_bonus", skill);
                    let bonus = get_attr_val_num_o(&self.current_attributes, key, main_id);
                    let total = (bonus + proficiency).to_string();

                    let rows = vec![proficiency.to_string(), bonus.to_string(), total];
                    d100_table.add_row_with_label(label, rows);
                }
                self.d100_skill_table = Box::new(d100_table);
            }
            {
                let mut d20_table = DynamicTable::default();
                let column_labels = vec![
                    Label::new(GOV_CAMEL, GOV),
                    Label::new(PROFICIENCY_CAMEL, PROFICIENCY),
                    Label::new(BONUS_CAMEL, BONUS),
                    Label::new(TOTAL_CAMEL, TOTAL_CAMEL),
                ];
                d20_table.add_column_labels(column_labels);
                for skill in D20_SKILLS.iter() {
                    let label = Label::new(skill, skill);

                    let key = format!("d20_skill_{}_proficiency", skill);
                    let proficiency = get_attr_val_num_o(&self.current_attributes, key, main_id);
                    let key = format!("d20_skill_{}_bonus", skill);
                    let bonus = get_attr_val_num_o(&self.current_attributes, key, main_id);
                    let key = format!("d20_skill_{}_governed_by", skill);
                    let gov = get_attr_val_str_o(&self.current_attributes, key, main_id);
                    let total = (bonus + proficiency).to_string();

                    let rows = vec![gov, proficiency.to_string(), bonus.to_string(), total];
                    d20_table.add_row_with_label(label, rows);
                }
                self.d20_skill_table = Box::new(d20_table);
            }
            {
                let mut resource_basic = DynamicTable::default();
                resource_basic.add_column_labels(vec![Label::new("Values", "")]);

                for basic in BASICS.iter() {
                    let label = Label::new(basic, basic);
                    let needle = basic.to_owned().to_owned();
                    let val = get_attr_val_str_o(&self.current_attributes, needle, main_id);
                    resource_basic.add_row_with_label(label, vec![val])
                }
                self.resources_basic = Box::new(resource_basic);
            }
            // const POINTS: [&str; 7] = [
            //     "Flair",
            //     "Surge",
            //     "Strain",
            //     "MP pool",
            //     "MP daily",
            //     "Ki pool",
            //     "Ki daily"
            //     "Psi daily",
            // ]
            {
                let mut resource_points = DynamicTable::default();
                self.resources_points = Box::new(resource_points);
            }
            // const BODY_PARTS: [&str; 8] = [
            //     "Head",
            //     "Neck",
            //     "Left Arm",
            //     "Right Arm",
            //     "Body",
            //     "Groin",
            //     "Left Leg",
            //     "Right Leg",
            // ];
            {
                let mut resource_body_hp = DynamicTable::default();
                self.resources_body_hp = Box::new(resource_body_hp);
            }

            self.images = imagemap;
            self.current = Some(loaded);
        }
        Ok(())
    }

    // Reset an image.
    pub(super) fn set_image(
        dbs: &mut Option<LoadedDbs>,
        image: &mut Option<dbimg::Image>,
        imagemap: &mut FnvHashMap<Option<i64>, egui_extras::RetainedImage>,
        name: String,
        uuid: String,
        id: i64,
        path: std::path::PathBuf,
    ) -> Result<(), String> {
        if let Some(ref mut dbs) = dbs {
            let input = dbimg::InputImage {
                of: id,
                link: path.into_os_string().into_string().map_err(ma)?,
            };
            let output = dbs.create_update_image(name, uuid, input)?;
            let processed = process_image(&output)?;
            *image = Some(output);
            imagemap.insert(Some(id), processed);
        }
        Ok(())
    }

    pub(super) fn update_main(
        dbs: &mut Option<LoadedDbs>,
        part: CharacterPart,
    ) -> Result<(), String> {
        if let Some(ref mut dbs) = dbs {
            let name = part.name().to_owned();
            let uuid = part.uuid().to_owned();
            dbs.create_update_part(part, (name, uuid))?;
        }
        Ok(())
    }

    // Update attributes.
    pub(super) fn update_attrs(
        dbs: &mut Option<LoadedDbs>,
        part: &mut CompleteCharacter,
        rows: &[Row],
    ) -> Result<(), String> {
        if let Some(ref mut dbs) = dbs {
            for r in rows.iter() {
                if let Some((ref mut k, ref mut v)) = part
                    .attributes_mut()
                    .iter_mut()
                    .find(|(k, _)| k.key() == r.label)
                {
                    match r.value.parse() {
                        Ok(v1) if Some(v1) != v.value_num() => {
                            v.update_value_num_by_ref(Some(v1));
                            dbs.create_update_attribute(
                                k.to_owned(),
                                v.to_owned(),
                                (part.name.to_owned(), part.uuid().to_owned()),
                            )?;
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }
}

/// This function exists for dry.
fn get_attr_val_num(
    attrs: &FnvHashMap<AttributeKey, AttributeValue>,
    needle: &str,
    of: i64,
) -> i64 {
    attrs
        .get(&AttributeKey::new(needle.to_owned(), of))
        .expect(&format!("{} is there.", needle))
        .value_num()
        .unwrap_or_default()
}
/// This function exists for dry.
fn get_attr_val_num_o(
    attrs: &FnvHashMap<AttributeKey, AttributeValue>,
    needle: String,
    of: i64,
) -> i64 {
    attrs
        .get(&AttributeKey::new(needle, of))
        .expect("Owned num attribute is there.")
        .value_num()
        .unwrap_or_default()
}
/// This function exists for dry.
fn get_attr_val_str_o(
    attrs: &FnvHashMap<AttributeKey, AttributeValue>,
    needle: String,
    of: i64,
) -> String {
    attrs
        .get(&AttributeKey::new(needle, of))
        .expect("Owned text attribute is there.")
        .value_text()
        .clone()
        .unwrap_or_default()
}

fn process_image(image: &dbimg::Image) -> Result<egui_extras::RetainedImage, String> {
    let ret = egui_extras::RetainedImage::from_image_bytes(image.of.to_string(), &image.content)
        .map_err(ma)?;
    Ok(ret)
}
