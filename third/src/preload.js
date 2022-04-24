const { contextBridge, ipcRenderer } = require('electron');


contextBridge.exposeInMainWorld('connection', {
  make: (event, arg) => ipcRenderer.invoke('connection:make', arg),
  send: (event, arg) => ipcRenderer.invoke('connection:send', arg),
  receive: (event, arg) => ipcRenderer.invoke('connection:receive', arg),
  get_system: (event, arg) => ipcRenderer.invoke('connection:get-system', arg),
  get_list: (event, arg) => ipcRenderer.invoke('connection:get-list', arg),
  get_sheet: (event, arg) => ipcRenderer.invoke('connection:get-sheet', arg),
  get_new_note: (event, arg) => ipcRenderer.invoke('connection:get-new-note', arg),
});

// A function that exists for DRY.
function clear_table(table) {
  table.deleteTHead();
  let len = table.rows.length;
  for(let i=len-1;i>=0;--i) {
    table.deleteRow(i);
  }
}

function create_cell(row, element) {
  let cell = row.insertCell();
  cell.appendChild(element);
}

contextBridge.exposeInMainWorld('builder', {
  character_list: (data) => {
    let table = document.getElementById('character-table');
    // Clear the old elements od the table if set. //;
    clear_table(table);
    let thead = table.createTHead();
    // Create elements.
    let row = table.insertRow();
    create_cell(row, document.createTextNode("^-^"));
    let el = document.createElement("INPUT");
    el.id = "create-character-input";
    create_cell(row, el);

    el = document.createElement('BUTTON');
    el.id = "create-character-button";
    el.innerText = "Create New Character";
    create_cell(row, el);
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
      let btn = document.createElement("BUTTON");
      btn.id = element["name"]+"load";
      btn.innerText = "Load "+element["name"];
      create_cell(row, btn);
    }
    ////////////////////////////////
    // Create header
    row = thead.insertRow();
    let headings = Object.keys(data[0]);
    for(let i=0;i<2;i++) {
      let th = document.createElement("th");
      let text = document.createTextNode(headings[i]);
      th.appendChild(text);
      row.appendChild(th);
    }
    let th = document.createElement("th");
    th.appendChild(document.createTextNode("Character Loader"));
    row.appendChild(th);
    ////////////////////////////////////
  },
  character_set: (character) => {
    document.getElementById('hide-main-wrap').hidden = false;
    document.getElementById('hide-resources-wrap').hidden = false;
    document.getElementById('hide-skills-wrap').hidden = false;
    document.getElementById('hide-inventory-wrap').hidden = false;
    set_main(character);
    set_level_table(character);
    set_main_attributes(character);
    set_main_attributes_cosmetic(character);
    set_main_attributes_resources(character);
    set_body_attributes(character);
    set_inventory(character);
    set_notes(character);

    set_d20_skills(character);
    set_d100_skills(character);
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
      let th = document.createElement("th");
      th.appendChild(document.createTextNode(x));
      row.appendChild(th);
    }
    ////////////////////////////////////
    row = thead.insertRow();
    for(let x of ["name","speed","weight","size","hp_current","hp_total"]) {
      let tbox = document.createElement("INPUT");
      tbox.id = x;
      tbox.value = char[x];
      create_cell(row, tbox);
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
      let th = document.createElement("th");
      th.appendChild(document.createTextNode(x));
      row.appendChild(th);
    }
    ////////////////////////////////////
    row = table.insertRow();
    for(let x of ["strength","reflex","toughness","endurance",
                  "intelligence","judgement","charm","will"]) {
      let tbox = document.createElement("INPUT");
      tbox.id = x + "input";
      tbox.value = char.attributes.find(att => att[0].key == x)[1].value_num;
      create_cell(row, tbox);
    }
    ////////////////////////////////////
    row = table.insertRow();
    for(let x of ["strength","reflex","toughness","endurance",
                  "intelligence","judgement","charm","will"]) {
      let el = document.createElement('SPAN');
      let val = (document.getElementById(x + 'input').value - 10) / 2;
      el.innerText = val;
      el.id = x + 'bonus';
      create_cell(row, el);
    }
}

function set_level_table(char) {
    let table = document.getElementById('level-table');
    clear_table(table);

    let thead = table.createTHead();
    let row = thead.insertRow();
    for(let x of ["Level", "Proficiency"]) {
      let th = document.createElement("th");
      th.appendChild(document.createTextNode(x));
      row.appendChild(th);
    }
    ///////////////////////
    row = table.insertRow();
    for(let x of ["Level","Proficiency"]) {
      let el = document.createElement('INPUT');
      el.value = char.attributes.find(att => att[0].key == x)[1].value_num;
      el.id = x;
      create_cell(row, el);
    }
}

function set_main_attributes_cosmetic(char) {
    let table = document.getElementById('character-cosmetic');
    clear_table(table);
    // Create elements.
    // Create header
    for(let x of ["Race", "Alignment", "Height", "Hair", "Eyes", "Age", "Skin", "Player"]) {
      let row = table.insertRow();
      let cell = row.insertCell();
      cell.appendChild(document.createTextNode(x));

      let tbox = document.createElement("INPUT");
      tbox.id = x + "input";
      tbox.value = char.attributes.find(att => att[0].key == x)[1].value_text;
      create_cell(row, tbox);
    }
}

function set_main_attributes_core(char) {
    let table = document.getElementById('character-cosmetic');
    clear_table(table);
    // Create elements.
    // Create header
    for(let x of ["Race", "Alignment", "Height", "Hair", "Eyes", "Age", "Skin", "Player"]) {
      let row = table.insertRow();
      let cell = row.insertCell();
      cell.appendChild(document.createTextNode(x));

      let tbox = document.createElement("INPUT");
      tbox.id = x + "input";
      tbox.value = char.attributes.find(att => att[0].key == x)[1].value_text;
      create_cell(row, tbox);
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
      let range_box = document.createElement("th");
      let min = attributes.find(att => att[0].key == "hit_min")[1].value_num;
      let max = attributes.find(att => att[0].key == "hit_max")[1].value_num;
      create_cell(row, document.createTextNode(min + ' - ' + max));

      // Create HP
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
      let ac_cell = document.createElement("INPUT");
      let key_ac = "armour";
      let val_ac = attributes.find(att => att[0].key == key_ac)[1].value_num;
      ac_cell.value = val_ac;
      ac_cell.id = key_ac + s.character_type;
      create_cell(row, ac_cell);
    }
  }
  let thead = table.createTHead();
  for (let x of ["Body part", "Hit-range", "Hitpoints", "Armour"]) {
    let th = document.createElement("th");
    th.appendChild(document.createTextNode(x));
    thead.appendChild(th);
  }
}

/// This function sets the d20 skills table.
/// The three components are 'd20_skill_'+'skill_name'+(`proficiency`/'bonux'/'governed_by')
function set_d20_skills(ch) {
  let table = document.getElementById('d20-skills');
  let attributes = ch.attributes;
  let prof_val = attributes.find(a => a[0].key == "Proficiency")[1].value_num;
  clear_table(table);
  for(let s of [["awareness","reflex"], ["acting","charm"], ["agility","reflex"],
                ["beast_mastery","judgement"], ["convince","charm"],["cunning","charm"],
                ["faith","judgement"], ["intuition","judgement"], ["knowledge","intelligence"],
                ["scrutiny","intelligence"], ["strong_arm","strength"], ["stealth","reflex"],
                ["survival","judgement"], ["trickery","reflex"]]) {
    let row = table.insertRow();
    // Name + roll
    let btn = document.createElement("BUTTON");
    btn.id = s[0]+"-roll";
    btn.innerText = s[0];
    create_cell(row, btn);

    // Governed by.
    let total = (document.getElementById(s[1] + 'input').value - 10) / 2;
    create_cell(row, document.createTextNode(s[1]));

    // Proficient.
    let npt = document.createElement('INPUT');
    npt.type = "checkbox";
    let attr_key = "d20_skill_"+s[0]+"_proficiency";
    let v = attributes.find(att => att[0].key == attr_key)[1].value_num;
    if(v && v > 0) { npt.checked = true } else { npt.checked = false };
    if(v > 0) { v = 1; }
    total += v * prof_val;
    npt.id = attr_key;
    create_cell(row, npt);

    // Extra bonus.
    npt = document.createElement('INPUT');
    attr_key = "d20_skill_"+s[0]+"_bonus";
    npt.id = attr_key;
    v = attributes.find(att => att[0].key == attr_key)[1].value_num;
    total += v;
    npt.value = v;
    create_cell(row, npt);

    // Total
    npt = document.createElement('SPAN');
    npt.innerText = total;
    npt.id = s[0]+'total';
    create_cell(row, npt);
    // cell.appendChild(document.createTextNode(total));
  }
  let thead = table.createTHead();
  let row = thead.insertRow();
  for(let t of ["Skill", "Governed by..", "Proficient", "Bonus", "Total"]) {
    let th = document.createElement("th");
    th.appendChild(document.createTextNode(t));
    row.appendChild(th);
  };
}

/// This function sets the d200 skill table.
/// The three components are 'd100_skill_'+'skill_name'+('proficiency'/'bonus')
function set_d100_skills(ch) {
  console.log("In `set_d100_skills`");
  let table = document.getElementById('d100-skills');
  let attributes = ch.attributes;
  clear_table(table);
  for(let s of ["armourer", "biomedicine", "combat_medicine", "demolition", "engineering", "firearms",
                "hacking", "melee", "piloting", "research", "surgery", "unarmed", "underworld"]) {
    let row = table.insertRow();
    // Name + roll
    let btn = document.createElement("BUTTON");
    btn.id = s+"-roll";
    btn.innerText = s;
    create_cell(row, btn);

    // Proficiency
    let npt = document.createElement('INPUT');
    attr_key = "d100_skill_"+s+"_proficiency";
    npt.id = attr_key;
    let val = attributes.find(att => att[0].key == attr_key)[1].value_num;
    npt.value = val;
    let total = val;
    create_cell(row, npt);

    // Extra bonus.
    npt = document.createElement('INPUT');
    attr_key = "d100_skill_"+s+"_bonus";
    npt.id = attr_key;
    val = attributes.find(att => att[0].key == attr_key)[1].value_num;
    npt.value = val;
    total += val;
    create_cell(row, npt);

    // Total.
    npt = document.createElement('SPAN');
    npt.innerText = total;
    npt.id = s+'total';
    console.log(npt);
    create_cell(row, npt);
  }
  let thead = table.createTHead();
  let row = thead.insertRow();
  for(let t of ["Skill", "Proficiency", "Bonus", "Total"]) {
    let th = document.createElement("th");
    th.appendChild(document.createTextNode(t));
    row.appendChild(th);
  };
}

function set_inventory(char) {
  // The aim of this is to create a table is to show basic information on
  // all inventory types items.
  let table = document.getElementById('character-inventory');
  clear_table(table);
  // Create main body.
  for (let item of char.parts) {
    if(item.part_type === "InventoryItem") {
      let row = table.insertRow();
      let gen_input = document.createElement('INPUT');
      let id = item.uuid;
      // Item Name
      gen_input.value = item.name;
      gen_input.id = 'item-name' + id;
      create_cell(row, gen_input);
      // Item Type.
      gen_input = document.createElement('INPUT');
      gen_input.value = item.character_type;
      gen_input.id = 'item-type' + id;
      create_cell(row, gen_input);
      // Item Size.
      gen_input = document.createElement('INPUT');
      gen_input.value = item.size;
      gen_input.id = 'item-size' + id;
      create_cell(row, gen_input);
      // Item weight.
      gen_input = document.createElement('INPUT');
      gen_input.value = item.weight;
      gen_input.id = 'item-weight' + id;
      create_cell(row, gen_input);
      // Item weight.
      gen_input = document.createElement('BUTTON');
      gen_input.innerText = 'Delete item';
      gen_input.id = 'delete' + id;
      create_cell(row, gen_input);
    }
  }
  let row = table.insertRow();
  let btn = document.createElement('BUTTON');
  btn.innerText = 'Add item';
  btn.id = 'add-inventory-item';
  create_cell(row, btn);

  // Create header.
  let thead = table.createTHead();
  row = thead.insertRow();
  for(let t of ["Item", "Item type", "Item size", "Item weight", ""]) {
    let th = document.createElement("th");
    th.appendChild(document.createTextNode(t));
    row.appendChild(th);
  };
}

function set_notes(char) {
  let table = document.getElementById('character-notes');
  clear_table(table);
  // Create elements.
  let row = table.insertRow();
  let btn = document.createElement("BUTTON");
  btn.id = "create-note";
  btn.innerText = "Create New Note";
  create_cell(row, btn);
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

  let thead = table.createTHead();
  thead.id = "hide-notes";
  row = thead.insertRow();
  let th = document.createElement("th");
  th.appendChild(document.createTextNode("Character Notes."));
  row.appendChild(th);
}

// All of the Node.js APIs are available in the preload process.
// It has the same sandbox as a Chrome extension.
window.addEventListener('DOMContentLoaded', () => {
  //
})
