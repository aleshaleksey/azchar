-- Create the root table.
create table character_dbs(
	id INTEGER primary key AUTOINCREMENT,
	name TEXT NOT NULL,
	uuid TEXT NOT NULL,
	db_path TEXT NOT NULL
);


create table permitted_attributes(
	id INTEGER primary key AUTOINCREMENT,
	key TEXT NOT NULL,
	attribute_type INTEGER NOT NULL,
	attribute_description TEXT NOT NULL
);

create table permitted_parts(
	id INTEGER primary key AUTOINCREMENT,
	part_name TEXT NOT NULL,
	-- Should be an enum which is shared with characters..
	part_type INTEGER NOT NULL,
);
