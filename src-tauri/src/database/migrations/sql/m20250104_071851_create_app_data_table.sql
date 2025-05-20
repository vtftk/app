CREATE TABLE IF NOT EXISTS "app_data" (
	"id"	INTEGER NOT NULL PRIMARY KEY,
	"data"	jsonb_text NOT NULL,
	"created_at"	datetime_text NOT NULL,
	"last_modified_at"	datetime_text NOT NULL
);