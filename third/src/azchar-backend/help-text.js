// This text is there to get people info about requests.
var requests_text = "azchar-server requests:\
\nRequest {\
\n    CreateSystem(String, String, String),\
\n    InitialiseFromPath(String),\
\n    CreateCharacterSheet(String),\
\n    CreateUpdateCharacter(CompleteCharacter),\
\n    UpdateAttribute(String, String, AttributeKey, AttributeValue),\
\n    CreateAttribute(String, String, InputAttribute),\
\n    UpdatePart(String, String, CharacterPart),\
\n    CreatePart(String, String, InputCharacter),\
\n    InsertUpdateImage(String, String, InputImage),\
\n    InsertNote(String, String, InputNote),\
\n    UpdateNote(String, String, Note),\
\n    DeleteCharacter(String, String),\
\n    ListCharacters,\
\n    LoadCharacter(String, String),\
\n    Roll(String),\
\n    Shutdown,\
\n    Invalid(String),\
\n}\n";
