CREATE TABLE IF NOT EXISTS "secrets" (
	"key"	VARCHAR NOT NULL PRIMARY KEY,
	"value"	VARCHAR NOT NULL,
	"metadata"	json_text NOT NULL,
	"created_at"	datetime_text NOT NULL
);