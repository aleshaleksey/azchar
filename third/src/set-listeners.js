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
  document.getElementById('hide-console-wrap').addEventListener('click', async () => {
    console.log("hide console clicked.");
    for(let x of ['input-request','submit-request','output-request']) {
      let el = document.getElementById(x);
      el.hidden = !el.hidden;
    }
  });
  for(let x of [
    ['hide-attacks-wrap','character-attacks'],
    ['hide-specials-wrap','character-specials'],
    ['hide-spells-wrap','character-spells'],
    ['hide-perks-wrap','character-perks'],
    ['hide-inventory-wrap','character-inventory'],
    ['hide-notes-wrap','character-notes'],
  ]) {
    document.getElementById(x[0]).addEventListener('click', async () => {
      console.log("hide inventory header clicked");
      let table = document.getElementById(x[1]);
      table.hidden = !table.hidden;
      let len = table.rows.length;
      if(table.tHead) { table.tHead.hidden = table.hidden; }
      for(let i=len-1;i>=1;--i) {
        table.rows[i].hidden = table.hidden;
      }
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



module.exports = {
  set_create_hide_listeners,
  set_roll_dialog_listener
};
