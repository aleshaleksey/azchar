const { contextBridge, ipcRenderer } = require('electron');
const {
  set_create_hide_listeners,
  set_roll_dialog_listener
} = require('./set-listeners.js');
const SetE = require('./set-elements.js');
const {
  D20_SKILL_LIST,
  D100_SKILL_LIST,
} = require('./constants.js');

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
  d20_skill_list: () => {
    return D20_SKILL_LIST;
  },
  d100_skill_list: () => {
    return D100_SKILL_LIST;
  },
  character_list: (data) => {
    SetE.set_character_list(data)
  },
  character_set: (character, reset) => {
    SetE.set_main(character);
    SetE.set_level_table(character);
    SetE.set_main_attributes(character);
    SetE.set_main_attributes_cosmetic(character);
    SetE.set_main_attributes_resources(character);
    SetE.set_body_attributes(character);
    SetE.set_subpart_table(character, 'character-attacks', "Ability", 'attack');
    SetE.set_subpart_table(character, 'character-specials', "Ability", 'special_ability');
    SetE.set_subpart_table(character, 'character-spells', "Ability", 'spell');
    SetE.set_subpart_table(character, 'character-perks', "Ability", 'perk');
    SetE.set_subpart_table(character, 'character-inventory', "InventoryItem");
    SetE.set_notes(character);
    //
    SetE.set_d20_skills(character);
    SetE.set_d100_skills(character);
    if(reset) {
      for(let x of ['hide-sheets-wrap','hide-main-wrap','hide-resources-wrap',
      'hide-skills-wrap','hide-notes-wrap','hide-attacks-wrap','hide-specials-wrap',
      'hide-spells-wrap','hide-perks-wrap','hide-inventory-wrap','character-main',
      'portrait-box','main-attributes-stats','level-table','hide-console-wrap']) {
        document.getElementById(x).hidden = false;
      }
      for(let x of ['character-table','d20-skills','d100-skills','main-body-parts','character-cosmetic','main-attributes-resources','character-inventory',
      'character-notes','character-attacks','character-specials','character-specials',
      'character-spells','character-perks','input-request','submit-request',
      'output-request']) {
        document.getElementById(x).hidden = true;
      }
    }
  },
  image_set: (part, portrait_id, size) => {
    SetE.set_portrait(part, portrait_id, size);
  },
  prepare_attr_update: () => {
    prepare_attr_update();
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
  roll_window_complex: (text) => {
    let box = document.getElementById('rr-box');
    box.hidden = false;
    box.innerText = text;
  },
  set_create_hide_listeners: () => {
    set_create_hide_listeners();
  },
  set_create_subpart_table: (part_type, part_subtype) => {
    SetE.create_new_part_table(part_type, part_subtype)
  },
  set_inventory_details: (part) => {
    // This deals with pthe part itself.
    SetE.set_inventory_details(part);
    // This deals with the part attributes.
    SetE.set_part_details(part);
    // This should set the part portrait.
    SetE.set_portrait(part, 'ip', 128);
    // This should let us set just the box.
    SetE.set_part_blurb_box(part);
  },
  set_roll_dialog_listener: () => {
    set_roll_dialog_listener();
  },
});

// All of the Node.js APIs are available in the preload process.
// It has the same sandbox as a Chrome extension.
window.addEventListener('DOMContentLoaded', () => {
  //
})
