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

function set_sheet_list_listeners(sheets, character) {
  if(!sheets) { return; }
  if(sheets.length > 0) {
    for (let char of sheets) {
      document.getElementById(char["name"]+"load").addEventListener('click', async () => {
        console.log("We have: " + char["name"] + "load");
        await window.connection.send(
          'click',
          "{\"LoadCharacter\":[\""+char["name"]+"\",\""+char["uuid"]+"\"]}"
        );
        await new Promise(r => setTimeout(r, 10));

        // Then we set the character sheet.
        let character = await window.connection.get_sheet('click', '');
        if(character) {
          console.log("Character in main: " + character["name"]);
          await window.builder.character_set(character);
          set_create_hide_listeners();
          set_create_note_listener(character);
          set_update_notes_listeners(character["name"], character["uuid"], character["notes"]);
          set_update_skills_listeners(character);
          set_update_main_attributes_listeners(character);
          set_update_main_attributes_cosmetic_listeners(character);
          set_update_main_attributes_resource_listeners(character);
          set_update_main_attributes_body_listeners(character);
          set_update_skills_listeners(character);
          set_skill_rollers(character);
          // TODO: Listeners for character sheet: Main part:
          set_update_main_listeners_for(character, ["name","speed","weight","size","hp_current","hp_total"]);
        }
      })
    }
  }
}

function set_create_hide_listeners() {
  document.getElementById('hide-main-wrap').addEventListener('click', async () => {
    console.log("hide main wrap clicked.");
    for(let x of ["character-main","level-table","main-attributes-stats","character-cosmetic",
                  "main-body-parts","main-attributes-resources"]) {

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
    console.log("hide notes header clicked");
    let table = document.getElementById('character-inventory');
    let len = table.rows.length;
    for(let i=len-1;i>=1;--i) {
      table.rows[i].hidden = !table.rows[i].hidden;
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
    await window.builder.character_set(ch);
    set_create_note_listener(ch);
    set_update_notes_listeners(ch);
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
  for(let s of ["awareness","acting","agility","beast_mastery","convince","cunning",
                "faith","intuition","knowledge","scrutiny","strong_arm","stealth",
                "survival","trickery"]) {
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
  for(let s of ["armourer", "biomedicine", "combat_medicine", "demolition", "engineering",
                "firearms", "hacking", "melee", "piloting", "research", "surgery",
                "unarmed", "underworld"]) {
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
  for(let s of ["awareness","acting","agility","beast_mastery","convince","cunning",
                "faith","intuition","knowledge","scrutiny","strong_arm","stealth",
                "survival","trickery"]) {
    document.getElementById(s+'-roll').addEventListener('click', async () => {
      console.log("pressed: "+s+"-roll");
      let v = Number.parseInt(document.getElementById(s+'total'));
      let roll;
      if(isNaN(v)) {
        roll = "{\"Roll\":\"1d20\"}";
      } else {
        roll = "{\"Roll\":\"1d20+"+v+"\"}";
      }
      console.log(roll);
      await window.connection.send('click', roll);
    });
  }
  for(let s of ["armourer", "biomedicine", "combat_medicine", "demolition", "engineering",
                "firearms", "hacking", "melee", "piloting", "research", "surgery",
                "unarmed", "underworld"]) {
    document.getElementById(s+'-roll').addEventListener('click', async () => {
      console.log("pressed: "+s+"-roll");
      let v = Number.parseInt(document.getElementById(s+'total'));
      let roll;
      if(isNaN(v)) {
        roll = "{\"Roll\":\"1d100+5\"}";
      } else {
        roll = "{\"Roll\":\"1d100\"+"+v+"}";
      }
      console.log(roll);
      await window.connection.send('click', roll);
    });
  }
}

// Set listeners for base character stats.
function set_update_main_attributes_listeners(ch) {
  // Set them.
  for(let x of ["strength","reflex","toughness","endurance",
                "intelligence","judgement","charm","will"]) {
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
}

function set_inventory_item_listeners(part, ch) {
  // Text values.
  for(let inner of ["name", "character_type", "size"]) {
    let el = document.getElementById(inner + part.uuid);
    el.addEventListener('keyup', async () => {
        part[inner] = el.value;
        await update_character_part(connection, ch, part);
    })
  }
  // Num values.
  let el = document.getElementById("weight" + part.uuid);
  el.addEventListener('keyup', async () => {
    console.log(el);
    console.log(el.value);
    if(el.value) {
      part.weight = el.value;
    } else {
      part.weight = null;
    };
    await update_character_part(connection, ch, part);
  })
  // Deletion of items.
  let eld = document.getElementById('delete' + part.uuid);
  eld.addEventListener('click', async () => {
    console.log("We pretend to delete the inventory item!");
  })
  // Deletion of items.
  let eli = document.getElementById('add-inventory-item');
  eli.addEventListener('click', async () => {
    console.log("We pretend to create the inventory item!");
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
      // This is a guard against invalid values.
      let intermediate = null;
      if(part.value.length > 0) {
        intermediate = part.value;
      }
      ch[x] = intermediate;
      await update_character_part(connection, ch, ch);
    });
  }
}
