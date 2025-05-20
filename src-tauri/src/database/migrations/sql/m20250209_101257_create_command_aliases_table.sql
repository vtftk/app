CREATE TABLE IF NOT EXISTS "command_alias" (
	"id"	uuid_text NOT NULL PRIMARY KEY,
	"command_id"	uuid_text NOT NULL,
	"alias"	VARCHAR NOT NULL,
	"order"	INTEGER NOT NULL,
	
    -- Connect to commands table
	FOREIGN KEY("command_id") 
        REFERENCES "commands" ("id")
        ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE INDEX "idx-command-alias" 
ON "command_alias" ("alias");