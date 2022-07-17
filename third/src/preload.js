const { contextBridge, ipcRenderer } = require('electron');
const {
  clear_table,
  create_cell,
  set_th,
  set_input,
  set_button,
  set_span
} = require('./preload-bp.js');
const {
  set_create_hide_listeners,
  set_roll_dialog_listener
} = require('./set-listeners.js');

contextBridge.exposeInMainWorld('connection', {
  make: (event, arg) => ipcRenderer.invoke('connection:make', arg),
  send: (event, arg) => ipcRenderer.invoke('connection:send', arg),
  receive: (event, arg) => ipcRenderer.invoke('connection:receive', arg),
  get_system: (event, arg) => ipcRenderer.invoke('connection:get-system', arg),
  get_list: (event, arg) => ipcRenderer.invoke('connection:get-list', arg),
  get_sheet: (event, arg) => ipcRenderer.invoke('connection:get-sheet', arg),
  get_new_note: (event, arg) => ipcRenderer.invoke('connection:get-new-note', arg),
  get_roll_res: (event, arg) => ipcRenderer.invoke('connection:get-roll-res', arg),
});

contextBridge.exposeInMainWorld('builder', {
  character_list: (data) => {
    let table = document.getElementById('character-table');
    // Clear the old elements od the table if set. //;
    clear_table(table);
    let thead = table.createTHead();
    // Create elements.
    let row = table.insertRow();
    create_cell(row, document.createTextNode("^-^"));

    set_input(row, "create-character-input", "")
    set_button(row, "create-character-button", "Create New Character");

    if(!data) { return; }
    if(data.length === 0) { return; }
    // For created characters
    for (let element of data) {
      row = table.insertRow();
      // Insert 'id'.
      let text = document.createTextNode(element["id"]);
      create_cell(row, text);
      // Insert 'name'
      text = document.createTextNode(element["name"]);
      create_cell(row, text);
      // Insert 'name'
      set_button(row, element["name"]+"load", "Load "+element["name"]);
      set_button(row, element["name"]+"delete", "Delete ");
    }
    ////////////////////////////////
    // Create header
    row = thead.insertRow();
    let headings = Object.keys(data[0]);
    for(let i=0;i<2;i++) {
      set_th(row, headings[i]);
    }
    set_th(row, "Character Loader");
    ////////////////////////////////////
  },
  character_set: (character, reset) => {
    set_main(character);
    set_level_table(character);
    set_main_attributes(character);
    set_main_attributes_cosmetic(character);
    set_main_attributes_resources(character);
    set_body_attributes(character);
    set_subpart_table(character, 'character-attacks', "Ability", 'attack');
    set_subpart_table(character, 'character-specials', "Ability", 'special_ability');
    set_subpart_table(character, 'character-spells', "Ability", 'spell');
    set_subpart_table(character, 'character-perks', "Ability", 'perk');
    set_subpart_table(character, 'character-inventory', "InventoryItem");
    set_notes(character);
    //
    set_d20_skills(character);
    set_d100_skills(character);
    if(reset) {
      for(let x of ['hide-sheets-wrap','hide-main-wrap','hide-resources-wrap',
      'hide-skills-wrap','hide-notes-wrap','hide-attacks-wrap','hide-specials-wrap',
      'hide-spells-wrap','hide-perks-wrap','hide-inventory-wrap','character-main',
      'main-attributes-stats','level-table','hide-console-wrap']) {
        document.getElementById(x).hidden = false;
      }
      for(let x of ['character-table','d20-skills','d100-skills','main-body-parts',
      'character-cosmetic','main-attributes-resources','character-inventory',
      'character-notes','character-attacks','character-specials','character-specials',
      'character-spells','character-perks','input-request','submit-request',
      'output-request']) {
        document.getElementById(x).hidden = true;
      }
    }
  },
  set_create_subpart_table: (part_type, part_subtype) => {
    create_new_part_table(part_type, part_subtype)
  },
  set_inventory_details: (part) => {
    // This deals with pthe part itself.
    set_inventory_details(part);
    // This deals with the part attributes.
    set_part_details(part);
  },
  // This function creates the sort of not-quite popup display with the roll.
  roll_window_100: (rolled_item, description, roll) => {
    let box = document.getElementById('rr-box');
    box.hidden = false;
    let thr = Number.parseInt(roll[1]);
    if(thr < 5) { thr = 5; }
    box.innerText = 'We rolled: ' + rolled_item + '\n'
      + description + ':\n'
      + 'Roll [' + roll[0] + '] vs Threshold [' + thr + ']';
  },
  // This function creates the sort of not-quite popup display with the roll.
  roll_window_20: (rolled_item, description, roll) => {
    let box = document.getElementById('rr-box');
    box.hidden = false;
    let res = Number.parseInt(roll[0]) + Number.parseInt(roll[1]);
    box.innerText = 'We rolled: ' + rolled_item + '\n'
      + description + ':\n'
      + 'Roll = ' + res + ' ([' + roll[0] + '] + ' + roll[1] + ')';
  },
  set_create_hide_listeners: () => {
    set_create_hide_listeners();
  },
  prepare_attr_update: () => {
    prepare_attr_update();
  },
  set_roll_dialog_listener: () => {
    set_roll_dialog_listener();
  }
});

function set_main(char) {
    let table = document.getElementById('character-main');
    clear_table(table);

    let thead = table.createTHead();
    let row = thead.insertRow();
    // Create elements.
    // Create header
    for(let x of ["Name","Speed","Weight","Size","HP","HP Total"]) {
      set_th(row, x);
    }
    ////////////////////////////////////
    row = thead.insertRow();
    set_input(row, "name", char.name);
    document.getElementById("name").disabled = true;
    for(let x of ["speed","weight","size","hp_current","hp_total"]) {
      set_input(row, x, char[x]);
    }
}

function set_main_attributes(char) {
    let table = document.getElementById('main-attributes-stats');
    clear_table(table);

    let thead = table.createTHead();
    let row = thead.insertRow();
    // Create elements.
    // Create header
    for(let x of ["STR","REF","TOU","END","INT","JUD","CHA","WIL"]) {
      set_th(row, x);
    }
    ////////////////////////////////////
    row = table.insertRow();
    for(let x of ["Strength","Reflex","Toughness","Endurance",
                  "Intelligence","Judgement","Charm","Will"]) {
      let val = char.attributes.find(att => att[0].key == x)[1].value_num;
      set_input(row, x + "input", val);
    }
    ////////////////////////////////////
    row = table.insertRow();
    for(let x of ["Strength","Reflex","Toughness","Endurance",
                  "Intelligence","Judgement","Charm","Will"]) {
      let val = (document.getElementById(x + 'input').value - 10) / 2;
      set_span(row, x + 'bonus', val);
    }
}

function set_level_table(char) {
    let table = document.getElementById('level-table');
    clear_table(table);

    let thead = table.createTHead();
    let row = thead.insertRow();
    for(let x of ["Level", "Proficiency"]) {
      set_th(row, x);
    }
    ///////////////////////
    row = table.insertRow();
    for(let x of ["Level","Proficiency"]) {
      set_input(row, x, char.attributes.find(att => att[0].key == x)[1].value_num);
    }
}

function set_main_attributes_cosmetic(char) {
    let table = document.getElementById('character-cosmetic');
    clear_table(table);
    // Create elements.
    // Create header
    for(let x of ["Race", "Alignment", "Height", "Hair", "Eyes", "Age", "Skin", "Player"]) {
      let row = table.insertRow();
      create_cell(row, document.createTextNode(x));
      let value = char.attributes.find(att => att[0].key == x)[1].value_text;
      set_input(row, x + "input", value);
    }
}

function set_main_attributes_core(char) {
    let table = document.getElementById('character-cosmetic');
    clear_table(table);
    // Create elements.
    // Create header
    for(let x of ["Race", "Alignment", "Height", "Hair", "Eyes", "Age", "Skin", "Player"]) {
      let row = table.insertRow();
      create_cell(row, document.createTextNode(x));
      let value = char.attributes.find(att => att[0].key == x)[1].value_text;
      set_input(row, x + "input", value);
    }
}

function set_main_attributes_resources(char) {
  let table = document.getElementById('main-attributes-resources');
  clear_table(table);

  // make most of the things.
  for(let a of [["Flair", "flair_current", "flair_maximum"],["Surge", "surge_current","surge_maximum"],
                ["MP Well", "mp_current", "mp_maximum"],["MP daily", "mp_use_day", "mp_use_day_max"],
                ["Ki Well", "ki_current", "ki_maximum"],["Psi daily", "psi_use_day","psi_use_day_max"],
                ["Strain", "strain"]]) {
    let row = table.insertRow();
    create_cell(row, document.createTextNode(a[0]));
    // NB: complex input that cannot be made with `set_input`.
    let npt = document.createElement('INPUT');
    npt.id = a[1];
    npt.value = char.attributes.find(att => att[0].key == a[1])[1].value_num;

    let cell = row.insertCell();
    cell.appendChild(npt);

    if(a.length > 2) {
      cell.appendChild(document.createTextNode("/"));
      npt = document.createElement('INPUT');
      npt.id = a[2];
      npt.value = char.attributes.find(att => att[0].key == a[2])[1].value_num;
      cell.appendChild(npt);
    }
  }
}

// Set what all the body parts do.
function set_body_attributes(ch) {
  console.log("In `set_body_attributes`.");
  let table = document.getElementById("main-body-parts");
  clear_table(table);
  for(let s of ch["parts"]) {
    if(s.part_type === "Body") {
      let attributes = s.attributes;
      // Create the row label.
      let row = table.insertRow();
      create_cell(row, document.createTextNode(s.character_type));

      // Create hit-range
      let min = attributes.find(att => att[0].key == "hit_min")[1].value_num;
      let max = attributes.find(att => att[0].key == "hit_max")[1].value_num;
      create_cell(row, document.createTextNode(min + ' - ' + max));

      // Create HP (NB: complex input that cannot be made with `set_input`)
      let hp_c = document.createElement("INPUT");
      let key_c = "hitpoints_current";
      let val_c = attributes.find(att => att[0].key == key_c)[1].value_num;
      hp_c.value = val_c;
      console.log("value of hp_c = " + hp_c.value);
      hp_c.id = key_c + s.character_type;

      let hp_m = document.createElement("INPUT");
      let key_m = "hitpoints_maximum";
      let val_m = attributes.find(att => att[0].key == key_m)[1].value_num;
      hp_m.value = val_m;
      hp_m.id = key_m + s.character_type;

      let hp_cell = row.insertCell();
      hp_cell.appendChild(hp_c);
      hp_cell.appendChild(document.createTextNode('/'));
      hp_cell.appendChild(hp_m);

      // Create Toughness
      // TODO: Add the attribute in the `cjfusion.toml` file!
      // Create armour.
      let val_ac = attributes.find(att => att[0].key == "armour")[1].value_num;
      set_input(row, "armour" + s.character_type, val_ac);
    }
  }
  let thead = table.createTHead();
  for (let x of ["Body part", "Hit-range", "Hitpoints", "Armour"]) {
    set_th(thead, x);
  }
}

/// This function sets the d20 skills table.
/// The three components are 'd20_skill_'+'skill_name'+(`proficiency`/'bonux'/'governed_by')
function set_d20_skills(ch) {
  let table = document.getElementById('d20-skills');
  let attributes = ch.attributes;
  let prof_val = attributes.find(a => a[0].key == "Proficiency")[1].value_num;
  clear_table(table);
  for(let s of [["Awareness","Reflex"], ["Acting","Charm"], ["Agility","Reflex"],
                ["Beast Mastery","Judgement"], ["Convince","Charm"],["Cunning","Charm"],
                ["Faith","Judgement"], ["Intuition","Judgement"], ["Knowledge","Intelligence"],
                ["Scrutiny","Intelligence"], ["Strong Arm","Strength"], ["Stealth","Reflex"],
                ["Survival","Judgement"], ["Trickery","Reflex"]]) {
    let row = table.insertRow();
    set_button(row, s[0]+"-roll", s[0]);

    // Governed by.
    let total = (document.getElementById(s[1] + 'input').value - 10) / 2;
    create_cell(row, document.createTextNode(s[1]));

    // Proficient.
    let attr_key = "d20_skill_"+s[0]+"_proficiency";
    let v = attributes.find(att => att[0].key == attr_key)[1].value_num;
    let checked = false;
    if(v && v > 0) { checked = true };
    if(v > 0) { v = 1; }
    total += v * prof_val;
    set_input(row, attr_key, checked, "checkbox");

    // Extra bonus.
    attr_key = "d20_skill_"+s[0]+"_bonus";
    v = attributes.find(att => att[0].key == attr_key)[1].value_num;
    total += v;
    set_input(row, attr_key, v);
    set_span(row, s[0]+'total', total);
  }
  let thead = table.createTHead();
  let row = thead.insertRow();
  for(let t of ["Skill", "Governed by..", "Proficient", "Bonus", "Total"]) {
    set_th(row, t);
  };
}

/// This function sets the d200 skill table.
/// The three components are 'd100_skill_'+'skill_name'+('proficiency'/'bonus')
function set_d100_skills(ch) {
  console.log("In `set_d100_skills`");
  let table = document.getElementById('d100-skills');
  let attributes = ch.attributes;
  clear_table(table);
  for(let s of ["Armourer", "Biomedicine", "Combat Medicine", "Demolition", "Engineering", "Firearms",
                "Hacking", "Melee", "Piloting", "Research", "Surgery", "Unarmed", "Underworld"]) {
    let row = table.insertRow();
    set_button(row, s+"-roll", s);

    // Proficiency
    attr_key = "d100_skill_"+s+"_proficiency";
    let val = attributes.find(att => att[0].key == attr_key)[1].value_num;
    let total = val;
    set_input(row, attr_key, val);

    // Extra bonus.
    attr_key = "d100_skill_"+s+"_bonus";
    val = attributes.find(att => att[0].key == attr_key)[1].value_num;
    total += val;
    set_input(row, attr_key, val);
    set_span(row, s+'total', total);
  }
  let thead = table.createTHead();
  let row = thead.insertRow();
  for(let t of ["Skill", "Proficiency", "Bonus", "Total"]) {
    set_th(row, t);
  };
}

/// This function sets a table for owned parts, such as inventory, spells, attacks etc.
/// `char`: Character object.
/// `table_id`: String giving the table id (eg 'character-inventory').
/// `part_type`: String (eg 'InventoryItem').
/// `part_subtype`: String giving the part subtype (eg 'weapon'). Optional Arg.
function set_subpart_table(char, table_id, part_type, part_subtype) {
  // The aim of this is to create a table is to show basic information on
  // all inventory types items.
  let table = document.getElementById(table_id);
  clear_table(table);
  // Create main body.
  for (let item of char.parts) {
    if(item.part_type === part_type) {
      if(!part_subtype || item.character_type==part_subtype) {
        let row = table.insertRow();
        let id = item.uuid;

        set_span(row, 'name' + id, item.name);
        console.log(item);
        set_span(row, 'character_type' + id, item.character_type);
        set_span(row, 'size' + id, item.size);
        let w = 0;
        if(item.weight) {
          w +=item.weight;
        }
        set_span(row, 'weight' + id, w);
        // Item weight.
        set_button(row, 'delete' + id, 'Delete');
      }
    }
  }
  let row = table.insertRow();
  set_button(row, 'add-to-'+table_id, 'Add new..');
  console.log(document.getElementById('add-to-'+table_id));

  // Create header.
  let thead = table.createTHead();
  row = thead.insertRow();
  for(let t of ["Item", "Item type", "Item size", "Item weight", ""]) {
    set_th(row, t);
  };
}

/// This function sets a table for owned parts, such as inventory, spells, attacks etc.
/// `part_type`: String (eg 'InventoryItem').
/// `part_subtype`: String giving the part subtype (eg 'weapon'). Optional Arg.
function create_new_part_table(part_type, part_subtype) {
// The item creation is set here.
  let item_creation_table = document.getElementById("item-creation-table");
  clear_table(item_creation_table);
  // Main rows
  let row = item_creation_table.insertRow();
  set_input(row, 'name-new', 'Spanky');
  // Create the type options.
  let select = document.createElement("SELECT");
  select.id = 'type-new';
  if(part_subtype!='tool') {
    console.log("Creating for :"+part_subtype);
    let option = new Option(part_subtype, part_subtype);
    option.selected = true;
    select.options.add(option);
  } else {
    console.log("Creating for inventory: "+part_subtype);
    for(let t of ['weapon', 'armour', 'tool', 'consumable', 'wealth']) {
      let option = new Option(t, t);
      if(t == 'tool') {
        option.selected = true;
      }
      select.options.add(option);
    }
  }
  create_cell(row, select);
  set_input(row, 'size-new', 'small');
  set_input(row, 'weight-new', 1);
  // Create/cancel buttons.
  row = item_creation_table.insertRow();
  set_button(row,'addInventoryItemYes', 'Create');
  set_button(row,'addInventoryItemNo', 'Cancel');
  // Header
  let thead = item_creation_table.createTHead();
  row = thead.insertRow();
  for(let t of ["Item", "Item type", "Item size", "Item weight"]) {
    set_th(row, t);
  };
}

// Sets inventory details when an item is clicked.
function set_inventory_details(part) {
  let table = document.getElementById("part-detail-table");
  {
    clear_table(table);
    // Main rows
    let row = table.insertRow();
    set_th(row, 'Name');
    set_input(row, 'name-detail', part.name);
    row = table.insertRow();
    set_th(row, 'Type');
    // Create the type options.
    let select = document.createElement("SELECT");
    select.id = 'character_type-detail';
    if(part.type!="InventoryItem") {
      let option = new Option(part.character_type, part.character_type);
      option.selected = true;
      select.options.add(option);
    } else {
      for(let t of ['weapon', 'armour', 'tool', 'consumable', 'wealth']) {
        let option = new Option(t, t);
        if(t == part.character_type) {
          option.selected = true;
        }
        select.options.add(option);
      }
    }
    console.log("new select: "+select);
    create_cell(row, select);
    row = table.insertRow();
    set_th(row, 'Size');
    set_input(row, 'size-detail', part.size);
    row = table.insertRow();
    set_th(row, 'Weight');
    set_input(row, 'weight-detail', part.weight);
    row = table.insertRow();
    set_th(row, 'Speed');
    set_input(row, 'speed-detail', part.speed);
    row = table.insertRow();
    set_th(row, 'HP total');
    set_input(row, 'hp_total-detail', part.hp_total);
    row = table.insertRow();
    set_th(row, 'HP current');
    set_input(row, 'hp_current-detail', part.hp_current);
    // Set the roll buttons.
    row = table.insertRow();
    set_button(row,'roll'+part.character_type+'-melee','Roll Melee');
    set_button(row,'roll'+part.character_type+'-ranged','Roll Ranged');

    // Header
    let thead = table.createTHead();
    row = thead.insertRow();
    set_th(row, "Characteristic");
    set_th(row, "Value");
  }
}

// Sets inventory details when an item is clicked.
// `part`: Is a character part.
// `tool`,
function set_part_details(part) {
  let table = document.getElementById("part-attribute-table");
  {
    // This is only necessary if we're dealing with a weapon.
    // NB: attribute keys are in the format of PARTTYPE_ATTRIBUTENAME.
    // Therefore by subtracting PARTTYPE_, you get the name.
    table.hidden = false;
    let pl = part.character_type.length + 1;
    console.log(pl);
    clear_table(table);
    let key = '';
    // Main rows
    for(let x of part.attributes.filter(x => x[0].key!="Blurb")) {
      let kl = x[0].key.length;
      let row = table.insertRow();
      set_th(row, x[0].key.substring(pl,kl), x[0].key);
      set_input(row, x[0].key+'-value-num', x[1].value_num);
      set_input(row, x[0].key+'-value-text', x[1].value_text);
    }

    // Header
    let thead = table.createTHead();
    row = thead.insertRow();
    set_th(row, "Stat");
    set_th(row, "Value");
    set_th(row, "Description");

    let blurb = part.attributes.find(x => x[0].key==="Blurb");
    if(blurb) { // We know blurb is some, but just incase.
      let box = document.getElementById('blurb-box');
      box.innerText = blurb[1].value_text;
    }
  }
}

function set_notes(char) {
  let table = document.getElementById('character-notes');
  clear_table(table);
  // Create elements.
  let row = table.insertRow();
  set_button(row, "create-note", "Create New Note");

  for (let x of char["notes"]) {
    let note_title = document.createElement("INPUT");
    note_title.id = "note-title" + x["id"];
    note_title.value = x["title"];
    row = table.insertRow();
    let cell = row.insertCell();
    cell.appendChild(note_title);
    cell.appendChild(document.createTextNode(x["id"]+": "+x["date"]));
    /////
    let note_c = document.createElement("TEXTAREA");
    note_c.class = "notebox";
    note_c.id = "note-content" + x["id"];
    note_c.value = x["content"];
    row = table.insertRow();
    create_cell(row, note_c);
  };
}

// All of the Node.js APIs are available in the preload process.
// It has the same sandbox as a Chrome extension.
window.addEventListener('DOMContentLoaded', () => {
  //
})
