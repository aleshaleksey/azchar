// Modules to control application life and create native browser window
const { app, BrowserWindow, ipcMain } = require('electron');
const path = require('path');
const { socket, create_socket, response, flow_controller } = require('./azchar-backend/socket-mod.js');
const { FlowController } = require('./azchar-backend/flow-control.js');
// const { create_socket } = require('./preload.js');
const {requests_text} = require('./azchar-backend/help-text.js');
const {ReplyKey, process_reply} = require('./azchar-backend/request-reply.js');

const createWindow = () => {
  // Create the browser window.
  let reload_path = path.join(__dirname, 'preload');
  reload_path = path.join(reload_path, 'mod.js');
  const mainWindow = new BrowserWindow({
    width: 1200,
    height: 900,
    backgroundColor: "#998876",
    color: "#220505",
    webPreferences: {
      preload: reload_path
    }
  });

  // and load the index.html of the app.
  mainWindow.loadFile('./src/index.html');
  mainWindow.webContents.openDevTools();
  // Open the DevTools.
  var socket = undefined;

  ipcMain.handle('connection:make', (event, arg) => {
    console.log("Make arg: " + arg);
    socket = create_socket(arg, true);
  });
  ipcMain.handle('connection:send', (event, arg) => {
    console.log("Make arg: " + arg);
    socket.send(arg);
  });

  ipcMain.handle('connection:receive', (event, arg) => {
    return response.text;
  });
  ipcMain.handle('connection:get-list', (event, arg) => {
    return flow_controller.sheets;
  });
  ipcMain.handle('connection:get-sheet', (event, arg) => {
    return flow_controller.character;
  });
  ipcMain.handle('connection:get-new-note', (event, arg) => {
    return flow_controller.new_note;
  });
  ipcMain.handle('connection:get-roll-res', (event, arg) => {
    return flow_controller.last_roll;
  });
}


// This method will be called when Electron has finished
// initialization and is ready to create browser windows.
// Some APIs can only be used after this event occurs.
app.whenReady().then(() => {
  createWindow();

  app.on('activate', () => {
    // On macOS it's common to re-create a window in the app when the
    // dock icon is clicked and there are no other windows open.
    if (BrowserWindow.getAllWindows().length === 0) createWindow()
  })
})

// Quit when all windows are closed, except on macOS. There, it's common
// for applications and their menu bar to stay active until the user quits
// explicitly with Cmd + Q.
app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') app.quit()
})

// In this file you can include the rest of your app's specific main process
// code. You can also put them in separate files and require them here.
