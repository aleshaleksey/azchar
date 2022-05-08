// This module exists for the niggly little functions that we want to hide
// from the main `preload.js` file, and yet do want to use a lot.

// A function that exists for DRY.
function clear_table(table) {
  table.deleteTHead();
  let len = table.rows.length;
  for(let i=len-1;i>=0;--i) {
    table.deleteRow(i);
  }
}

// Create a cell.
function create_cell(row, element) {
  let cell = row.insertCell();
  cell.appendChild(element);
}

// Set the stuff inside a th.
function set_th(row, text) {
  let th = document.createElement("th");
  th.appendChild(document.createTextNode(text));
  row.appendChild(th);
}

// Create an input.
// Takes an id, a vlue and a row to stick it in.
function set_input(row, id, value, npt_type) {
    let ibox = document.createElement("INPUT");
    ibox.id = id;
    ibox.value = value;
    if(npt_type) { ibox.type = npt_type; }

    create_cell(row, ibox);
}

function set_button(row, id, inner_text) {
  let btn = document.createElement("BUTTON");
  btn.id = id;
  btn.innerText = inner_text;
  create_cell(row, btn);
}

function set_span(row, id, inner_text) {
  let span = document.createElement('SPAN');
  span.innerText = inner_text;
  span.id = id;
  create_cell(row, span);
}

module.exports = { clear_table, create_cell, set_th, set_input, set_button, set_span };
