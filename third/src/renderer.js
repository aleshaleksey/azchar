// Connection listener.
var character;
var sheets = [];
// document.getElementById('submit-address').addEventListener('click', async () => {
//   await window.connection.make('click', document.getElementById('input-address').value);
//   document.getElementById('output-request').value = 'Connection established. Hopefully.';
// })
try {
  window.connection.make('click', 'ws://127.0.0.1:55555');
  document.getElementById('output-request').value = 'Connection established.';
} catch (e) {
  document.getElementById('output-request').value = e;
}

// Choose db.
document.getElementById('submit-system').addEventListener('click', async () => {
  await list_sheets();
  document.getElementById('hide-sheets-wrap').hidden = false;
  document.getElementById('character-table').hidden = false;
});

async function list_sheets() {
  await window.connection.send(
    'click',
    "{\"InitialiseFromPath\":\""+document.getElementById('input-system').value+"\"}"
  );
  await new Promise(r => setTimeout(r, 50));
  // Get character list.
  sheets = await window.connection.get_list('click', '');
  console.log(sheets);
  if(sheets && sheets.length != 0) {
    document.getElementById('display-system').innerText =
      sheets.length + " Characters in " + document.getElementById('input-system').value;
    // create character menu.
    await window.builder.character_list(sheets);
    // Get buttons and add loaders.
    set_sheet_list_listeners(sheets, character);
    set_create_character_listener();
  } else {
    await window.builder.character_list(sheets);
    set_create_character_listener();
  }
}

document.getElementById('submit-request').addEventListener('click', async () => {
  // const res = await window.connection.send('click', document.getElementById('input-request').value);
  await window.connection.send('click', document.getElementById('input-request').value);
  await new Promise(r => setTimeout(r, 30));
  document.getElementById('output-request').value = await window.connection.receive('click', '');
})

async function get_char_by_name_uuid(name, uuid, delay) {
  await window.connection.send(
    'click',
    "{\"LoadCharacter\":[\""+name+"\",\""+uuid+"\"]}"
  );
  await new Promise(r => setTimeout(r, delay));
  // Then we set the character sheet.
  let character = await window.connection.get_sheet('click', '');
  return character;
}

async function delete_character(name, uuid, delay) {
  await window.connection.send(
    'click',
    "{\"DeleteCharacter\":[\""+name+"\",\""+uuid+"\"]}"
  );
  await new Promise(r => setTimeout(r, delay));
}

function set_sheet_list_listeners(sheets) {
  if(!sheets) { return; }
  if(sheets.length > 0) {
    for (let char of sheets) {
      document.getElementById(char["name"]+"load").addEventListener('click', async () => {
        console.log("We have: " + char["name"] + "load");
        // Then we set the character sheet.
        let character = await get_char_by_name_uuid(char.name, char.uuid, 30);
        await set_all_listeners(character, true);
      })
      document.getElementById(char["name"]+"delete").addEventListener('click', async () => {
        console.log("We have: " + char["name"] + "delete");
        // Then we set the character sheet.
        await delete_character(char.name, char.uuid, 30);
        await list_sheets();
      })
    }
  }
}

async function prepare_attr_update(conn, el, ch, s, skill) {
   let sum = document.getElementById(s+'total');
   let sum_temp = 0;
   if(sum.innerText && !isNaN(sum.innerText)) {
     sum_temp = Number.parseInt(sum.innerText);
   }
   let n = Number.parseInt(el.value);
   if(isNaN(n)) { n = 0; }

   let a = ch.attributes.find(attr => attr[0].key == skill);
   if(!a[1].value_num || isNaN(a[1].value_num)) { a[1].value_num = 0; }
   sum_temp -= a[1].value_num;
   a[1].value_num = n;
   await update_attribute(conn, a[0], a[1], ch);
   sum_temp += a[1].value_num;
   sum.innerText = sum_temp;
}

async function set_all_listeners(ch, reset) {
  if(ch) {
    console.log("Character in main: " + ch["name"]);
    await window.builder.character_set(ch, reset);
    await window.builder.set_create_hide_listeners();
    set_create_note_listener(ch);
    set_update_notes_listeners(ch.name, ch.uuid, ch.notes);
    set_update_skills_listeners(ch);
    set_update_main_attributes_listeners(ch);
    set_update_main_attributes_cosmetic_listeners(ch);
    set_update_main_attributes_resource_listeners(ch);
    set_update_main_attributes_body_listeners(ch);
    set_update_skills_listeners(ch);
    set_skill_rollers(ch);
    await window.builder.set_roll_dialog_listener();
    // TODO: Listeners for character sheet: Main part:
    set_update_main_listeners_for(ch, ["name","speed","weight","size","hp_current","hp_total"]);
  } else {
    console.log("Could not set listeners as character is null.");
  }
}

// Creates a note, retrieves it, and resets the character.
function set_create_note_listener(ch) {
  document.getElementById('create-note').addEventListener('click', async () => {
    await window.connection.send(
      'click',
      "{\"InsertNote\":[\""+ch["name"]+"\",\""+ch["uuid"]+"\",{\"title\":\"\",\"content\":\"\"}]}"
    );
    await new Promise(r => setTimeout(r, 10));
    let n = await window.connection.get_new_note('click', '');
    while(!n) {
      await new Promise(r => setTimeout(r, 10));
      n = await window.connection.get_new_note('click', '');
    }
    let notes = [n];
    let l = ch.notes.length;
    for(let n of ch.notes) {
      notes.push(n);
    }
    ch.notes = notes;
    character = ch;
    await set_all_listeners(ch, false);
  })
}

function set_update_notes_listeners(name, uuid, notes) {
  if(!notes) { return; }
  if(notes < 1) { return; }
  for(let n of notes) {
    let title = document.getElementById("note-title" + n["id"]);
    let content = document.getElementById("note-content" + n["id"]);
    for(let el of [title, content]) {
      el.addEventListener('keyup', async () => {
        let c = content.value.replaceAll('\"','\'');
        n.content = c;
        n.title = title.value;
        c = c.replaceAll('\n','[[enter]]');
        await window.connection.send('keyup',
          "{\"UpdateNote\":[\""+name+"\",\""+uuid
          +"\",{\"id\":"+n["id"]
            +",\"date\":\""+n["date"]
            +"\",\"title\":\""+title.value
            +"\",\"content\":\""+c+"\"}]}"
        )
      })
    }
  }
}

// NB: A main character is a little bit different from a character part,
// So the inner updater is a little different.
function update_character_part(conn, ch, part) {
  let cht = "Main";
  if(part.part_type) {
    cht = part.part_type;
  }
  let belongs_to = null;
  if(part.belongs_to) {
    belongs_to = part.belongs_to;
  }
  conn.send(
    'keyup',
    "{\"UpdatePart\":[\""
      +ch["name"]+"\",\""
      +ch["uuid"]+"\","
      +"{\"id\":"+part.id
      +",\"name\":\""+part.name
      +"\",\"uuid\":\""+part.uuid
      +"\",\"character_type\":\""+part.character_type
      +"\",\"speed\":"+part.speed
      +",\"weight\":"+part.weight
      +",\"size\":\""+part.size
      +"\",\"hp_total\":"+part.hp_total
      +",\"hp_current\":"+part.hp_current
      +",\"part_type\":\""+cht
      +"\",\"belongs_to\":"+belongs_to
      +",\"attributes\":[],\"image\":"+part.image
      +"}]}"
  );
}

// The part type and name must be specified to make this universal.
// NB: `character_type` is a more or less free-form string, while
// `part_type` comes from a set selection of enums.
function create_character_part(
  conn,
  ch,
  character_type,
  part_type,
  part_name,
  part_size,
  part_weight) {
  conn.send(
    'keyup',
    "{\"CreatePart\":[\""
      +ch["name"]+"\",\""
      +ch["uuid"]+"\","
      +"{\"name\":\""+part_name
      +"\",\"character_type\":\""+character_type
      +"\",\"speed\":"+0
      +",\"weight\":"+part_weight
      +",\"size\":\""+part_size
      +"\",\"hp_total\":"+0
      +",\"hp_current\":"+0
      +",\"belongs_to\":"+ch.id
      +",\"part_type\":\""+part_type
      +"\"}]}"
  );
}

/// An inner function for DRY, as updating attributes is generally the same.
/// NB: `att_val` is the updated attribute value structure.
function update_attribute(conn, att_key, att_val, ch) {
  if(!att_val.value_num) {
    att_val.value_num = null;
  };
  conn.send(
    'keyup',
    "{\"UpdateAttribute\":[\""+ch["name"]+"\",\""+ch["uuid"]+"\",{\"key\":\""+att_key.key
    +"\",\"of\":"+att_key.of
    +"},{\"id\":"+att_val.id
    +",\"value_num\":"+att_val.value_num
    +",\"value_text\":\""+att_val.value_text
    +"\",\"description\":\""+att_val.description+"\"}]}"
  )
}

async function prepare_attr_update(conn, el, ch, s, skill) {
   let sum = document.getElementById(s+'total');
   let sum_temp = 0;
   if(sum.innerText && !isNaN(sum.innerText)) {
     sum_temp = Number.parseInt(sum.innerText);
   }
   let n = Number.parseInt(el.value);
   if(isNaN(n)) { n = 0; }

   let a = ch.attributes.find(attr => attr[0].key == skill);
   if(!a[1].value_num || isNaN(a[1].value_num)) { a[1].value_num = 0; }
   sum_temp -= a[1].value_num;
   a[1].value_num = n;
   await update_attribute(conn, a[0], a[1], ch);
   sum_temp += a[1].value_num;
   sum.innerText = sum_temp;
}

/// A function to set the value of the skill total. It is used both for updates
/// of the character part, and the actual sheet info.
///
/// `skill`: A string with the skill name.
/// `prof_check_box`: Check box input, (asking if skill has proficiency)
/// `ch`: Character object.
function set_d20_skill_total(skill, chkbx, ch) {
  let sum = document.getElementById(skill+'total');
  let gv = document.getElementById(skill+'gov').innerText;
  let s_val =  Number.parseInt((document.getElementById(gv+'input').value - 10) / 2);
  s_val += ch.attributes.find(att => att[0].key == "d20_skill_"+skill+"_bonus")[1].value_num;

  let val = 0
  if(chkbx.checked) {
    val = ch.attributes.find(attr => attr[0].key == "Proficiency")[1].value_num;
  }

  let a = ch.attributes.find(attr => attr[0].key == 'd20_skill_'+skill+'_proficiency');
  if(!a[1].value_num || isNaN(a[1].value_num)) { a[1].value_num = 0; }
  a[1].value_num = val;
  sum.innerText = a[1].value_num + s_val;
  return a;
}

// Set listeners for skills (d20 and d100 in one function)
function set_update_skills_listeners(ch) {
  for(let s of window.builder.d20_skill_list()) {
    let check = document.getElementById('d20_skill_'+s+'_proficiency');
    // Checkbox detects click.
    check.addEventListener('click', async () => {
      let a = set_d20_skill_total(s, check, ch);
      await update_attribute(connection, a[0], a[1], ch);
    });

    let el2 = document.getElementById('d20_skill_'+s+'_bonus');
    el2.addEventListener('keyup', async () => {
      prepare_attr_update(connection, el2, ch, s, 'd20_skill_'+s+'_bonus');
    });
  }
  /////////////////////////////////////////////////////////////////////////////////
  for(let s of window.builder.d100_skill_list()) {
    let el = document.getElementById('d100_skill_'+s+'_proficiency')
     el.addEventListener('keyup', async () => {
      prepare_attr_update(connection, el, ch, s, 'd100_skill_'+s+'_proficiency');
    });

    let el2 = document.getElementById('d100_skill_'+s+'_bonus');
    el2.addEventListener('keyup', async () => {
      prepare_attr_update(connection, el2, ch, s, 'd100_skill_'+s+'_bonus');
    });
  }
}

function set_skill_rollers(ch) {
  for(let s of window.builder.d20_skill_list()) {
    document.getElementById(s+'-roll').addEventListener('click', async () => {
      console.log("pressed: "+s+"-roll");
      let v = Number.parseInt(document.getElementById(s+'total').innerText);
      let roll;
      if(isNaN(v)) {
        roll = "{\"Roll\":\"1d20\"}";
      } else if(v >= 0) {
        roll = "{\"Roll\":\"1d20+"+v+"\"}";
      } else {
        roll = "{\"Roll\":\"1d20"+v+"\"}";
      }

      await window.connection.send('click', roll);
      await new Promise(r => setTimeout(r, 10));

      let res = await window.connection.get_roll_res();
      window.builder.roll_window_20(s, s + " roll result", res);
    });
  }
  for(let s of window.builder.d100_skill_list()) {
    document.getElementById(s+'-roll').addEventListener('click', async () => {
      console.log("pressed: "+s+"-roll");
      let v = Number.parseInt(document.getElementById(s+'total').innerText);
      console.log("bonus for d100:" + v);
      let roll;
      if(isNaN(v)) {
        roll = "{\"Roll\":\"1d100+5\"}";
      } else if(v >= 0) {
        roll = "{\"Roll\":\"1d100+"+v+"\"}";
      } else {
        roll = "{\"Roll\":\"1d100"+v+"\"}";
      }

      await window.connection.send('click', roll);
      await new Promise(r => setTimeout(r, 10));

      let res = await window.connection.get_roll_res('click');
      window.builder.roll_window_100(s, s + " roll result", res);
    });
  }
}

// Set listeners for base character stats.
function set_update_main_attributes_listeners(ch) {
  // Set them.
  for(let x of ["Strength","Reflex","Toughness","Endurance",
                "Intelligence","Judgement","Charm","Will"]) {
    let n = document.getElementById(x+'input');
    n.addEventListener('keyup', async () => {
      // Update the attributes.
      let a = ch.attributes.find(attr => attr[0].key == x);
      if(isNaN(n.value)) { n.value = 0; }
      a[1].value_num = n.value;
      document.getElementById(x+'bonus').innerText =
        (document.getElementById(x + 'input').value - 10) / 2;
      await update_attribute(connection, a[0], a[1], ch);
      // Update the skills table. NB: May slow things.
      for(let s of window.builder.d20_skill_list()) {
        let gv = document.getElementById(s+'gov').innerText;
        if(gv===x) {
          let check = document.getElementById('d20_skill_'+s+'_proficiency');
          set_d20_skill_total(s, check, ch);
        }
      }
    });
  }
}

// Update the listeners for some fairly basic things.
function set_update_main_attributes_cosmetic_listeners(ch) {
  for(let x of ["Race", "Alignment", "Height", "Hair", "Eyes", "Age", "Skin",
                "Player"]) {
    let el = document.getElementById(x+'input');
    // console.log(el);
    el.addEventListener('keyup', async () => {
      let a = ch.attributes.find(att => att[0].key == x);
      a[1].value_text = el.value;
      await update_attribute(connection, a[0], a[1], ch);
    })
  }
}

function set_update_main_attributes_resource_listeners(ch) {
  // make most of the things.
  for(let x of ["flair_current", "flair_maximum", "surge_current", "surge_maximum",
                "mp_current", "mp_maximum", "mp_use_day", "mp_use_day_max", "ki_current",
                "ki_maximum", "psi_use_day","psi_use_day_max", "strain",
                "Level", "Proficiency"]) {
    let el = document.getElementById(x);

    el.addEventListener('keyup', async () => {
      let a = ch.attributes.find(att => att[0].key == x);
      if(isNaN(el.value)) {
        el.value = 0;
      }
      if(!el.value) {
        a[1].value_num = 0;
      } else {
        a[1].value_num = Number.parseInt(el.value);
      }

      if(x==="Proficiency") {
        for(let s of window.builder.d20_skill_list()) {
          let check = document.getElementById('d20_skill_'+s+'_proficiency');
          set_d20_skill_total(s, check, ch);
        }
      }
      await update_attribute(connection, a[0], a[1], ch);
    })
  }
}

// NB: This also sets the inventory listeners.
function set_update_main_attributes_body_listeners(ch) {
  // Outer parts loop.
  for(let s of ch["parts"]) {
    if(s.part_type === "Body") {
      for(let inner of ["hitpoints_current", "hitpoints_maximum", "armour"]) {
        let el = document.getElementById(inner + s.character_type);
        el.addEventListener('keyup', async () => {
            let a = s.attributes.find(att => att[0].key == inner);
            if(el.value) {
              a[1].value_num = el.value;
            } else {
              a[1].value_num = null;
            };
            await update_attribute(connection, a[0], a[1], ch);
        })
      }
    } else if(s.part_type === "InventoryItem" || s.part_type === "Ability") {
      // We want to set this for attacks, spells, special abilities, inventory items,
      // but for now lets do it for everything.
      set_inventory_item_listeners(s, ch)
    }
  }
  // Creation of items.
  for(let x of [['character-attacks', "Ability", 'attack'],
  ['character-specials', "Ability", 'special_ability'],
  ['character-spells', "Ability", 'spell'],
  ['character-perks', "Ability", 'perk'],
  ['character-inventory', "InventoryItem", 'tool']]) {
    let table_id = x[0];
    let part_type = x[1];
    let subtype = x[2];
    let eli = document.getElementById('add-to-'+table_id);
    console.log("table_id:"+table_id);
    // console.log(eli);
    eli.addEventListener('click', async () => {
      // Create the creation table.
      await window.builder.set_create_subpart_table(part_type, subtype);
      let table = document.getElementById('item-box');
      table.hidden = false;

      document.getElementById('addInventoryItemNo').addEventListener(
        'click',
        async () => {
        table.hidden = true;
      });
      document.getElementById('addInventoryItemYes').addEventListener(
        'click',
        async () => {
        table.hidden = true;
        // Set parameters.
        let weight = document.getElementById('weight-new').value;
        let size = document.getElementById('size-new').value;
        let name = document.getElementById('name-new').value;
        // To do: Convert `itype` to lowercase.
        let sel = document.getElementById('type-new');
        let itype = sel.options[sel.selectedIndex].innerText;
        if(!weight) { weight = 0; }
        if(!size) { size = 'medium'; }
        if(!name) { name = 'Spanky'; }
        if(!itype) { itype = subtype; }
        // Create.
        await create_character_part(connection, ch, itype, part_type, name, size, weight);
        await new Promise(r => setTimeout(r, 100));
        ch = await window.connection.get_sheet('click', '');
        character = ch;
        await set_all_listeners(character, false);
      })
    });
  }
}

async function pseudo_update_inventory_item(part, ch) {
  window.builder.set_inventory_details(part);
  console.log("done");
  // Deal with the box.
  let box = document.getElementById('item-box-details');
  box.hidden = false;
  box.addEventListener('dblclick', async () => {
    // When closing the box, reload the character and have it updated.
    // TODO: Currently fails.
    document.getElementById("blurb-box").value = "";
    ch = await get_char_by_name_uuid(ch.name, ch.uuid, 50);
    await set_all_listeners(ch, false);
    box.hidden = true;
    character = ch;
  });

  // Text values.
  for(let inner of ['name', 'size']) {
    let el = document.getElementById(inner + '-detail');
    el.addEventListener('keyup', async () => {
        part[inner] = el.value;
        await update_character_part(connection, ch, part);
    });
  }
  // Character type is a selection which is a pain.
  let sel = document.getElementById('character_type-detail');
  console.log("retrieved select: "+sel);
  sel.addEventListener('change', async () => {
      part.character_type = sel.options[sel.selectedIndex].innerText;
      await update_character_part(connection, ch, part);
  });
  // Numerical values.
  for(let inner of ['weight', 'speed', 'hp_total',
                    'hp_current']) {
    let el = document.getElementById(inner+'-detail');
    el.addEventListener('keyup', async () => {
      if(el.value) {
        part[inner] = el.value;
      } else {
        part[inner] = null;
      };
      await update_character_part(connection, ch, part);
    });
  }
  // Add item listeners.
  console.log("about to cycle attributes.");
  // Most attributes are used in general.
  for(let x of part.attributes.filter(x => x[0].key!="Blurb")) {
    let el = document.getElementById(x[0].key+'-value-num');
    el.addEventListener('keyup', async () => {
      // console.log(el);
      if(el.value) {
        let n = Number.parseFloat(el.value);
        if(isNaN(n)) { n = 0; }
        x[1].value_num = n;
      } else {
        x[1].value_num = null;
      }
      await update_attribute(connection, x[0], x[1], ch);
    });
    let el2 = document.getElementById(x[0].key+'-value-text');
    el2.addEventListener('keyup', async () => {
      // console.log(el2);
      if(el2.value) {
        x[1].value_text = el2.value;
      } else {
        x[1].value_text = null;
      };
      await update_attribute(connection, x[0], x[1], ch);
    });
  }
    // Blurb is special!
  let blurb = part.attributes.find(x => x[0].key==='Blurb');
  console.log("part_name: "+part.name);
  console.log("Blurb text in update: "+blurb[1].value_text);
  if(blurb) {
    let bbox = document.getElementById("blurb-box");
    bbox.addEventListener('keyup', async () => {
      if(bbox.value) {
        blurb[1].value_text = bbox.value;
      } else {
        blurb[1].value_text = "Gimme blurb.";
      };
      await update_attribute(connection, blurb[0], blurb[1], ch);
    })
  }
}

function set_inventory_item_listeners(part, ch) {
  // For any of the normal buttons, open the detail dialog.
  for(let inner of ["name", "character_type", "size", "weight"]) {
    document.getElementById(inner + part.uuid).addEventListener(
      'click',
      async () => await pseudo_update_inventory_item(part, ch)
    );
  }

  // Deletion of items.
  let eld = document.getElementById('delete' + part.uuid);
  eld.addEventListener('click', async () => {
    console.log("ch.parts:" + ch.parts.length);
    await connection.send(
      'click',
      "{\"DeletePart\":[\""+ch["name"]+"\",\""+ch["uuid"]+"\","+part.id+"]}",
    );
    await new Promise(r => setTimeout(r, 100));

    ch = await window.connection.get_sheet('click', '');
    character = ch;
    await set_all_listeners(character, false);
  })
}

function set_create_character_listener() {
  document.getElementById('create-character-button').addEventListener('click', async () => {
    await connection.send(
      'click',
      "{\"CreateCharacterSheet\":\""+document.getElementById('create-character-input').value+"\"}",
    );
    await new Promise(r => setTimeout(r, 50));
    let sheets = await window.connection.get_list('click', '');
    await window.builder.character_list(sheets);
    set_sheet_list_listeners(sheets, character);
    set_create_character_listener();
  })
}

/// Listeners for main parts
function set_update_main_listeners_for(ch, text_array) {
  for (let x of text_array) {
    let part = document.getElementById(x);
    part.addEventListener('keyup', async () => {
      // This is a guard against invalid valuest.
      let intermediate = null;
      if(part.value.length > 0) {
        intermediate = part.value;
      }
      ch[x] = intermediate;
      await update_character_part(connection, ch, ch);
    });
  }
}

// This function hides and clears the roll window:
function set_roll_dialog_listener() {
  let box = document.getElementById('rr-box');
  box.addEventListener('dblclick', async () => {
    box.hidden = true;
    box.innerText = null;
  });
}
