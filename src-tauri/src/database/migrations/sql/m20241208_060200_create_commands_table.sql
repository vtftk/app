CREATE TABLE IF NOT EXISTS "commands" (
	"id"	uuid_text NOT NULL PRIMARY KEY,
	"enabled"	BOOLEAN NOT NULL,
	"name"	VARCHAR NOT NULL,
	"command"	TEXT NOT NULL,
	"config"	json_text NOT NULL,
	"order"	INTEGER NOT NULL DEFAULT 0,
	"created_at"	datetime_text NOT NULL,
);  

CREATE INDEX "idx-command-enabled" 
ON "commands" ("enabled");

CREATE INDEX "idx-command-command" 
ON "commands" ("command");