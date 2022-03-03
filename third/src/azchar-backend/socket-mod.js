const WebSocket = require('ws');
const {process_reply} = require('./request-reply.js');
const FlowController = require('./flow-control.js');

var response = { text: "No Responses received yet.", rec: false };
var socket = null;
var flow_controller = FlowController;
function create_socket(address, test) {
  console.log("Getting socket for address: " + address);
  let socket = new WebSocket(address);
  socket.onopen = function(e) {
      console.log("Socket is open.");
  };
  socket.onmessage = function(e) {
    if(test) {
      response.text = e.data;
      response.rec = true;
    }
    console.log("Message received.");
    flow_controller = process_reply(JSON.parse(e.data), flow_controller);
  };
  socket.onsend = function(e) {
    console.log("Sent a message");
  };
  socket.onclose = function(e) {
    console.log("Socket is closing, recreating...");
    socket = create_socket(address);
  };
  socket.onerror = function(e) {
    console.log("Socket error: " + e.message);
  };
  return socket;
}

function socket_send(data) {
  socket.send(data)
}

module.exports = { create_socket, socket_send, response, socket, flow_controller };
