CREATE TABLE IF NOT EXISTS "chat_history" (
	"id"	uuid_text NOT NULL PRIMARY KEY,
	"user_id"	VARCHAR NOT NULL,
	"message"	VARCHAR NOT NULL,
	"cheer"	INTEGER,
	"created_at" datetime_text NOT NULL
);