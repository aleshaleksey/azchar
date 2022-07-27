const {clear_table } = require('./set-elements-bp.js');

function set_create_hide_listeners() {
  document.getElementById('hide-main-wrap').onclick = async function(e) {
    console.log("hide main wrap clicked.");
    for(let x of ["portrait-box","character-main","level-table","main-attributes-stats"]) {
      let el = document.getElementById(x);
      el.hidden = !el.hidden;
    }
  };
  document.getElementById('hide-resources-wrap').onclick = async function(e) {
    console.log("hide resources clicked.");
    for(let x of ["character-cosmetic","main-attributes-resources", "main-body-parts"]) {
      let el = document.getElementById(x);
      el.hidden = !el.hidden;
    }
  };
  document.getElementById('hide-skills-wrap').onclick = async function(e) {
    console.log("hide skills clicked.");
    for(let x of ["d20-skills","d100-skills"]) {
      let el = document.getElementById(x);
      el.hidden = !el.hidden;
    }
  };
  document.getElementById('hide-console-wrap').onclick = async function(e) {
    console.log("hide console clicked.");
    for(let x of ['input-request','submit-request','output-request']) {
      let el = document.getElementById(x);
      el.hidden = !el.hidden;
    }
  };
  for(let x of [
    ['hide-attacks-wrap','character-attacks'],
    ['hide-specials-wrap','character-specials'],
    ['hide-spells-wrap','character-spells'],
    ['hide-perks-wrap','character-perks'],
    ['hide-inventory-wrap','character-inventory'],
    ['hide-notes-wrap','character-notes'],
    ['hide-sheets-wrap','character-table'],
  ]) {
      document.getElementById(x[0]).onclick = async function(e) {
        console.log('hide '+x[0]+'inventory header clicked');
        let table = document.getElementById(x[1]);
        table.hidden = !table.hidden;
        let len = table.rows.length;
        if(table.tHead) { table.tHead.hidden = table.hidden; }
        for(let i=len-1;i>=1;--i) {
          table.rows[i].hidden = table.hidden;
        }
      };
  }
}

// This function hides and clears the roll window:
function set_roll_dialog_listener() {
  let box = document.getElementById('rr-box');
  box.removeEventListener('dblclick', async () => {});
  box.addEventListener('dblclick', async () => {
    box.hidden = true;
    box.innerText = null;
  });
}


module.exports = {
  set_create_hide_listeners,
  set_roll_dialog_listener,
};
