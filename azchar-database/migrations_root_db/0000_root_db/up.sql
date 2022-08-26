-- Create the root table.
create table character_dbs(
	id INTEGER primary key AUTOINCREMENT,
	name TEXT NOT NULL,
	uuid TEXT NOT NULL,
	db_path TEXT NOT NULL
);

create table permitted_attributes(
	key TEXT NOT NULL,
	attribute_type INTEGER NOT NULL,
	attribute_description TEXT NOT NULL,
	part_name TEXT,
	default_value TEXT,
	part_type INTEGER,
	obligatory BOOLEAN NOT NULL,
	PRIMARY KEY(key, part_name, part_type)
);

create table permitted_parts(
	id INTEGER primary key AUTOINCREMENT,
	part_name TEXT NOT NULL,
	-- Should be an enum which is shared with characters..
	part_type INTEGER NOT NULL,
	obligatory BOOLEAN NOT NULL
);

create index if not exists attributes_idx on permitted_attributes(part_name, part_type);
create index if not exists parts_idx on permitted_parts(id);
