CREATE TABLE IF NOT EXISTS "events" (
	"id"	uuid_text NOT NULL PRIMARY KEY,
	"enabled"	BOOLEAN NOT NULL,
	"name"	VARCHAR NOT NULL,
    -- "trigger_type" is stored virtual column derived from the "type" discriminated union variant
    -- identifier for "trigger" used for searching based on type without needing to parse all the JSON
	"trigger_type"	VARCHAR GENERATED ALWAYS AS (JSON_EXTRACT("config", '$.trigger.type')) STORED,
	"config"	json_text NOT NULL,
	"order"	INTEGER NOT NULL DEFAULT 0,
	"created_at"	datetime_text NOT NULL
);

-- Index across the trigger type and enabled fields for faster lookups of event triggers
CREATE INDEX "idx-events-trigger-type-enabled" 
ON "events" ("trigger_type", "enabled");