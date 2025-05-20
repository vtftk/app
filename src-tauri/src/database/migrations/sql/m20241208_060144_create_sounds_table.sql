CREATE TABLE IF NOT EXISTS "sounds" (
	"id"	uuid_text NOT NULL PRIMARY KEY,
	"name"	VARCHAR NOT NULL,
	"src"	VARCHAR NOT NULL,
	"volume"	FLOAT NOT NULL,
	"order"	INTEGER NOT NULL DEFAULT 0,
	"created_at"	datetime_text NOT NULL
);