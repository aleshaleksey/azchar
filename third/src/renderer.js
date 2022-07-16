// Connection listener.
var character;
var sheets = [];
document.getElementById('submit-address').addEventListener('click', async () => {
  await window.connection.make('click', document.getElementById('input-address').value);
  document.getElementById('output-request').value = 'Connection established. Hopefully.';
})

// Choose db.
document.getElementById('submit-system').addEventListener('click', async () => {
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
})

document.getElementById('submit-request').addEventListener('click', async () => {
  // const res = await window.connection.send('click', document.getElementById('input-request').value);
  await window.connection.send('click', document.getElementById('input-request').value);
  await new Promise(r => setTimeout(r, 20));
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

function set_sheet_list_listeners(sheets) {
  if(!sheets) { return; }
  if(sheets.length > 0) {
    for (let char of sheets) {
      document.getElementById(char["name"]+"load").addEventListener('click', async () => {
        console.log("We have: " + char["name"] + "load");
        // Then we set the character sheet.
        let character = await get_char_by_name_uuid(char.name, char.uuid, 20);
        await set_all_listeners(character, true);
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

function set_create_hide_listeners() {
  document.getElementById('hide-main-wrap').addEventListener('click', async () => {
    console.log("hide main wrap clicked.");
    for(let x of ["character-main","level-table","main-attributes-stats"]) {
      let el = document.getElementById(x);
      el.hidden = !el.hidden;
    }
  });
  document.getElementById('hide-resources-wrap').addEventListener('click', async () => {
    console.log("hide resources clicked.");
    for(let x of ["character-cosmetic","main-attributes-resources", "main-body-parts"]) {
      let el = document.getElementById(x);
      el.hidden = !el.hidden;
    }
  });
  document.getElementById('hide-skills-wrap').addEventListener('click', async () => {
    console.log("hide skills clicked.");
    for(let x of ["d20-skills","d100-skills"]) {
      let el = document.getElementById(x);
      el.hidden = !el.hidden;
    }
  });
  document.getElementById('hide-inventory-wrap').addEventListener('click', async () => {
    console.log("hide inventory header clicked");
    let table = document.getElementById('character-inventory');
    table.hidden = !table.hidden;
    let len = table.rows.length;
    table.tHead.hidden = table.hidden;
    for(let i=len-1;i>=1;--i) {
      table.rows[i].hidden = table.hidden;
    }
  });
  document.getElementById('hide-notes').addEventListener('click', async () => {
    console.log("hide notes header clicked");
    let table = document.getElementById('character-notes');
    let len = table.rows.length;
    for(let i=len-1;i>=1;--i) {
      table.rows[i].hidden = !table.rows[i].hidden;
    }
  });
}

// Creates a note, retrieves it, and resets the character.
function set_create_note_listener(ch) {
  document.getElementById('create-note').addEventListener('click', async () => {
    await window.connection.send(
      'click',
      "{\"InsertNote\":[\""+ch["name"]+"\",\""+ch["uuid"]+"\",{\"title\":\"\",\"content\":\"\"}]}"
    );
    await new Promise(r => setTimeout(r, 5));
    let n = await window.connection.get_new_note('click', '');
    while(!n) {
      await new Promise(r => setTimeout(r, 5));
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

// Set listeners for skills (d20 and d100 in one function)
function set_update_skills_listeners(ch) {
  for(let s of ["Awareness","Acting","Agility","Beast Mastery","Convince","Cunning",
                "Faith","Intuition","Knowledge","Scrutiny","Strong Arm","Stealth",
                "Survival","Trickery"]) {
    let el = document.getElementById('d20_skill_'+s+'_proficiency');
    // Checkbox detects click.
    el.addEventListener('click', async () => {
      let sum = document.getElementById(s+'total');
      let sum_temp = 0;
      if(sum.innerText && !isNaN(sum.innerText)) {
        sum_temp = Number.parseInt(sum.innerText);
      }
      let val = 0
      if(el.checked) {
        val = ch.attributes.find(attr => attr[0].key == "Proficiency")[1].value_num;
      }

      let a = ch.attributes.find(attr => attr[0].key == 'd20_skill_'+s+'_proficiency');
      if(!a[1].value_num || isNaN(a[1].value_num)) { a[1].value_num = 0; }
      sum_temp -= a[1].value_num;
      a[1].value_num = val;
      await update_attribute(connection, a[0], a[1], ch);
      sum_temp += a[1].value_num;
      sum.innerText = sum_temp;
    });

    let el2 = document.getElementById('d20_skill_'+s+'_bonus');
    el2.addEventListener('keyup', async () => {
      prepare_attr_update(connection, el2, ch, s, 'd20_skill_'+s+'_bonus');
    });
  }
  /////////////////////////////////////////////////////////////////////////////////
  for(let s of ["Armourer", "Biomedicine", "Combat Medicine", "Demolition", "Engineering",
                "Firearms", "Hacking", "Melee", "Piloting", "Research", "Surgery",
                "Unarmed", "Underworld"]) {
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
  for(let s of ["Awareness","Acting","Agility","Beast Mastery","Convince","Cunning",
                "Faith","Intuition","Knowledge","Scrutiny","Strong Arm","Stealth",
                "Survival","Trickery"]) {
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
      await new Promise(r => setTimeout(r, 5));

      let res = await window.connection.get_roll_res();
      window.builder.roll_window_20(s, s + " roll result", res);
    });
  }
  for(let s of ["Armourer", "Biomedicine", "Combat Medicine", "Demolition", "Engineering",
                "Firearms", "Hacking", "Melee", "Piloting", "Research", "Surgery",
                "Unarmed", "Underworld"]) {
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
      await new Promise(r => setTimeout(r, 5));

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
      let a = ch.attributes.find(attr => attr[0].key == x);
      a[1].value_num = n.value;
      document.getElementById(x+'bonus').innerText =
        (document.getElementById(x + 'input').value - 10) / 2;
      await update_attribute(connection, a[0], a[1], ch);
    });
  }
}

// Update the listeners for some fairly basic things.
function set_update_main_attributes_cosmetic_listeners(ch) {
  for(let x of ["Race", "Alignment", "Height", "Hair", "Eyes", "Age", "Skin",
                "Player"]) {
    let el = document.getElementById(x+'input');
    console.log(el);
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
      if(el.value) {
        a[1].value_num = el.value;
      } else {
        a[1].value_num = null;
      };
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
    } else if(s.part_type === "InventoryItem") {
      set_inventory_item_listeners(s, ch)
    }
  }
  // Creation of items.
  let eli = document.getElementById("addInventoryItem");
  eli.addEventListener('click', async () => {
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
      let sel = document.getElementById('character_type-detail');
      let itype = sel.options[sel.selectedIndex].innerText;
      if(!weight) { weight = 0; }
      if(!size) { size = 'medium'; }
      if(!name) { name = 'Spanky'; }
      if(!itype) { itype = 'tool'; }
      // Create.
      await create_character_part(connection, ch, itype, "InventoryItem", name, size, weight);
      await new Promise(r => setTimeout(r, 100));
      ch = await window.connection.get_sheet('click', '');
      character = ch;
      await set_all_listeners(character, false);
    })
  })
}

async function pseudo_update_inventory_item(part, ch, inner) {
  console.log("Pretending to update "+inner);
  window.builder.set_inventory_details(part);
  console.log("done");
  // Deal with the box.
  let box = document.getElementById('item-box-details');
  box.hidden = false;
  box.addEventListener('dblclick', async () => {
    // When closing the box, reload the character and have it updated.
    // TODO: Currently fails.
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
  // Add weapon listeners.
  if(part.character_type == 'weapon') {
    for(let x of ['Range','Handedness','Categories','AP cost','Penetration','Damage type 1',
      'Damage type 2','Damage type 3','Damage 1','Damage 2','Damage 3','Attack bonus',
      'Description','Kind','Material','Value','Ammo type','Ammo count'
    ]) {
      let key = 'weapon_' + x;
      let attr = part.attributes.find(att => att[0].key == key)
      if(attr) {
        let el = document.getElementById(key+'-value-num');
        el.addEventListener('keyup', async () => {
          console.log(el);
          if(el.value) {
            let n = Number.parseFloat(el.value);
            if(isNaN(n)) { n = 0; }
            attr[1].value_num = n;
          } else {
            attr[1].value_num = null;
          }
          await update_attribute(connection, attr[0], attr[1], ch);
        });
        let el2 = document.getElementById(key+'-value-text');
        el2.addEventListener('keyup', async () => {
          if(el2.value) {
            attr[1].value_text = el2.value;
          } else {
            attr[1].value_text = null;
          };
          await update_attribute(connection, attr[0], attr[1], ch);
        });
      }
    }
  }
}

function set_inventory_item_listeners(part, ch) {
  // For any of the normal buttons, open the detail dialog.
  for(let inner of ["name", "character_type", "size", "weight"]) {
    document.getElementById(inner + part.uuid).addEventListener(
      'click',
      async () => await pseudo_update_inventory_item(part, ch, inner)
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
