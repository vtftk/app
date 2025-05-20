CREATE TABLE IF NOT EXISTS "model_data" (
	"id"	VARCHAR NOT NULL PRIMARY KEY,
    "name"	TEXT NOT NULL,
    "calibration"	json_text NOT NULL
);