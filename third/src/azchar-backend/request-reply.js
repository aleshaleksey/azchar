// Object.keys(Request) gets names of types.
// val==Request.CreateSystem.name, gets 'CreateSystem'
// Request.CreateSystem instanceof CreateSystem.
// class Request {
//     static CreateSystem = new Request('CreateSystem');
//     static InitialiseFromPath = new Request('InitialiseFromPath');
//     static CreateCharacterSheet = new Request('CreateCharacterSheet');
//     static CreateUpdateCharacter = new Request('CreateUpdateCharacter');
//     static UpdateAttribute = new Request('UpdateAttribute');
//     static CreateAttribute = new Request('CreateAttribute');
//     static UpdatePart = new Request('UpdatePart');
//     static CreatePart = new Request('CreatePart');
//     static InsertUpdateImage = new Request('InsertUpdateImage');
//     static InsertNote = new Request('InsertNote');
//     static UpdateNote = new Request('UpdateNote');
//     static DeleteCharacter = new Request('DeleteCharacter');
//     static ListCharacters = new Request('ListCharacters');
//     static LoadCharacter = new Request('LoadCharacter');
//     static Roll = new Request('Roll');
//     static Shutdown = new Request('Shutdown');
//     static Invalid = new Request('Invalid');
//
//     constructor(name) {
//       //
//     }
//     keyToString() {
//       return `$this.name`;
//     }
// };

// class Cre
class ReplyKey {
    static CreateSystem = new ReplyKey('CreateSystem');
    static InitialiseFromPath = new ReplyKey('InitialiseFromPath');
    static CreateCharacterSheet = new ReplyKey('CreateCharacterSheet');
    static CreateUpdateCharacter = new ReplyKey('CreateUpdateCharacter');
    static UpdateAttribute = new ReplyKey('UpdateAttribute');
    static UpdatePart = new ReplyKey('UpdatePart');
    static CreateAttributePart = new ReplyKey('CreateAttributePart');
    static InsertUpdateImage = new ReplyKey('InsertUpdateImage');
    static UpdateNote = new ReplyKey('UpdateNote');
    static InsertNote = new ReplyKey('InsertNote');
    static DeleteCharacter = new ReplyKey('DeleteCharacter');
    static ListCharacters = new ReplyKey('ListCharacters');
    static LoadCharacter = new ReplyKey('LoadCharacter');
    static Shutdown = new ReplyKey('Shutdown');
    static Roll = new ReplyKey('Roll');
    static Invalid = new ReplyKey('Invalid');
    static Err = new ReplyKey('Err');

    constructor(name) {
      this.name = name;
    }
    keyToString() {
      console.log(this.name);
      return '$this.name';
    }
};

// This function processes a reply. Currently it simply prints stuff,
// However, one mature it would set appropriate fields in the GUI in accordance
// with the reply.
function process_reply(json_input, flow) {
  console.log("Last known sender: " + flow.last_sender_id);
  // let test = ReplyKey.Invalid;
  // Deal with invalid replies.
  if (json_input[ReplyKey.Invalid.name]) {
    console.log("We have an invalid reply!");
    console.log(json_input[ReplyKey.Invalid.name]);
  // Deal with errors.
  } else if (json_input[ReplyKey.Err.name]) {
    console.log("Error: " + json_input[ReplyKey.Err.name][0]);
    console.log("Original content: " + json_input[ReplyKey.Err.name][1]);
  // deal with rolls.
  } else if (json_input[ReplyKey.Roll.name]) {
    console.log("Roll is: " + json_input[ReplyKey.Roll.name]);
  // Deal with Shutdown.
  } else if (json_input[ReplyKey.Shutdown.name]) {
    console.log("Shutting down the server.");
  } else if (json_input[ReplyKey.InsertNote.name]) {
    flow.new_note = json_input[ReplyKey.InsertNote.name];
    console.log("Note inserted: " + flow.new_note);
  // Deal with updating note.
  } else if (json_input[ReplyKey.UpdateNote.name]) {
    console.log("Note updated: " + json_input[ReplyKey.UpdateNote.name]);
  } else if (json_input[ReplyKey.CreateSystem.name]) {
    console.log("Created system: " + json_input[ReplyKey.CreateSystem.name]);
  } else if (json_input[ReplyKey.InsertUpdateImage.name]) {
    console.log("Inserted image.");
  } else if (json_input[ReplyKey.UpdateAttribute.name]) {
    console.log("Attribute updated.");
  // When reating an attribute or part we refresh the character.
  } else if (json_input[ReplyKey.CreateAttributePart.name]) {
    console.log("Major character update.");
    flow.character = json_input[ReplyKey.CreateAttributePart.name];
    console.log(flow.character);
  // Load the character.
  } else if (json_input[ReplyKey.LoadCharacter.name]) {
    console.log("Character loaded.");
    flow.character = json_input[ReplyKey.LoadCharacter.name];
    // console.log(flow.character);
    // console.log(flow.character.attributes[0]);
    // console.log(flow.character.parts[2].attributes);
  // Deal with Initialisation of DB.
  } else if (json_input[ReplyKey.InitialiseFromPath.name]) {
    console.log("Initialised the database.");
    flow.sheets = json_input[ReplyKey.InitialiseFromPath.name];
    console.log(flow.sheets);
  // Deal with Character deletion.
  } else if (json_input[ReplyKey.DeleteCharacter.name]) {
    console.log("Deleted character, remaining:");
    let l = json_input[ReplyKey.DeleteCharacter.name].length;
    for (let i=0; i<l;i++) {
      console.log(JSON.stringify(json_input[ReplyKey.DeleteCharacter.name][i]));
    }
    flow.sheets = json_input[ReplyKey.DeleteCharacter.name];
  // Deal with Character listing..
} else if (json_input[ReplyKey.ListCharacters.name]) {
    console.log("Listing Character:");
    let l = json_input[ReplyKey.ListCharacters.name].length;
    for (let i=0; i<l;i++) {
      console.log(JSON.stringify(json_input[ReplyKey.ListCharacters.name][i]));
    }
    flow.sheets = json_input[ReplyKey.ListCharacters.name];
  // Deal with creation of a character sheet..
  } else if (json_input[ReplyKey.CreateCharacterSheet.name]) {
    console.log("Created a sheet. Available sheets:");
    let l = json_input[ReplyKey.CreateCharacterSheet.name].length;
    for (let i=0; i<l;i++) {
      console.log(JSON.stringify(json_input[ReplyKey.CreateCharacterSheet.name][i]));
    }
    flow.sheets = json_input[ReplyKey.CreateCharacterSheet.name];
  // Deal with global update of a sheet.
  } else if (json_input[ReplyKey.CreateUpdateCharacter.name]) {
    console.log("Created a sheet. Available sheets:");
    let l = json_input[ReplyKey.CreateUpdateCharacter.name].length;
    for (let i=0; i<l;i++) {
      console.log(JSON.stringify(json_input[ReplyKey.CreateUpdateCharacter.name][i]));
    }
    flow.sheets = json_input[ReplyKey.CreateUpdateCharacter.name];
  // Deal wih note insertion.
} else if (JSON.stringify(json_input)=="\""+ReplyKey.UpdatePart.name+"\"") {
    console.log("Part updated.");
  } else if (JSON.stringify(json_input)=="\""+ReplyKey.UpdateAttribute.name+"\"") {
    console.log("Attribute updated.");
  } else if (JSON.stringify(json_input)=="\""+ReplyKey.UpdateNote.name+"\"") {
    console.log("Note updated: " + json_input[ReplyKey.UpdateNote.name]);
  } else {
    console.log("OH NO! We have: " + JSON.stringify(json_input));
  }
  return flow;
}
module.exports = { ReplyKey, process_reply };
