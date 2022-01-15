-- Create the root table
-- NB: Everything is a character, including, bodyparts, items and spells.
-- All things have a name, type, weight,
create table characters(
  -- The basic fields.
  id INTEGER primary key AUTOINCREMENT,
  name TEXT NOT NULL,
  uuid TEXT NOT NULL UNIQUE,
  character_type TEXT NOT NULL, -- NB: Text for maximum flexibility.
  -- Basic, vital attributes for characters.
  -- NB: Might not be relevant to all things.
  speed INTEGER NOT NULL,
  weight INTEGER,
  size TEXT,
  -- Not all things have hitpoints, mp. etc.
  hp_total INTEGER,
  hp_current INTEGER,
  -- What, if anything does this character belong to?
  belongs_to BIGINT references characters(id),
  -- What kind of part is it. See appropriate type.
  part_type INTEGER
);

-- A basic set of keys and values.
create table attributes(
  id INTEGER primary key AUTOINCREMENT,
  key TEXT NOT NULL,
  value_num INTEGER,
  value_text TEXT,
  description TEXT,
  of BIGINT NOT NULL references characters(id),
  UNIQUE(key, of)
);

create index if not exists attributes_idx on attributes(id);
create index if not exists attributes_ofx on attributes(of);
create index if not exists attributes_keyx on attributes(key);
create index if not exists characters_idx on characters(id);
create index if not exists characters_belonx on characters(belongs_to);
create index if not exists characters_belonx on characters(part_type);
