CREATE TABLE IF NOT EXISTS "event_executions" (
	"id"	uuid_text NOT NULL PRIMARY KEY,
	"event_id"	uuid_text NOT NULL,
	"metadata"	json_text NOT NULL,
	"created_at"	datetime_text NOT NULL,
    
	FOREIGN KEY ("event_id") 
        REFERENCES "events" ("id") 
        ON DELETE CASCADE ON UPDATE CASCADE
);