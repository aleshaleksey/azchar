# azchar
A TTRPG system-agnostic system for storing characters.
## Aims
The project aims to create an integrated "server" that facilitates the storing, loading, and manipulation of characters in a system agnostic fashion.
### Planned features
- Working App (working main loop, etc).
- System Creation from a TOML definition.
- Storage of a system file as a Sqlite DB.
- Character sheet creation from a system definition.
- Saving and loading of character sheet to separate Sqlite DBs referenced in the system file.
- Export and import of character sheets as JSON/TOML for interaction with frontends.
- Integrating dice roller based on libazdice.

### Currently working features.
- System Creation from a TOML definition.
- Storage of a system file as a Sqlite DB.
- Character sheet creation from a system definition.
- Saving and loading of character sheets (untested).

## Used 'Technologies'
- Rust
- Diesel (https://docs.rs/diesel/)
- libazdice. (https://github.com/aleshaleksey/libazdice)
