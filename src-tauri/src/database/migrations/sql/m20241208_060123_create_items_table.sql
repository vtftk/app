CREATE TABLE IF NOT EXISTS "items" (
	"id"	uuid_text NOT NULL PRIMARY KEY,
	"name"	VARCHAR NOT NULL,
	"order"	INTEGER NOT NULL DEFAULT 0,
	"config"	json_text NOT NULL,
    "created_at"	datetime_text
);