CREATE TABLE IF NOT EXISTS "command_logs" (
	"id"	        uuid_text NOT NULL PRIMARY KEY,
	"command_id"	uuid_text NOT NULL,
	"level"	        INTEGER NOT NULL,
	"message"	    VARCHAR NOT NULL,
	"created_at"	datetime_text NOT NULL,

    -- Connect to commands table
	FOREIGN KEY("command_id") 
        REFERENCES "commands" ("id") 
        ON DELETE CASCADE ON UPDATE CASCADE
);