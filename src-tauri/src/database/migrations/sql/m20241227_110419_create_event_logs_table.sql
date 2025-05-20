CREATE TABLE IF NOT EXISTS "event_logs" (
	"id"	uuid_text NOT NULL PRIMARY KEY,
	"event_id"	uuid_text NOT NULL,
	"level"	INTEGER NOT NULL,
	"message"	VARCHAR NOT NULL,
	"created_at"	datetime_text NOT NULL,

    -- Connect to events table
	FOREIGN KEY("event_id") 
        REFERENCES "events" ("id") 
        ON DELETE CASCADE ON UPDATE CASCADE
);