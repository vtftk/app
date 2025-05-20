CREATE TABLE IF NOT EXISTS "items_sounds" (
	"item_id"	uuid_text NOT NULL,
	"sound_id"	uuid_text NOT NULL,
	"sound_type"	 TEXT NOT NULL,

    -- Junction table uses a composite key of the item, sound id and sound type combined
	CONSTRAINT "pk_items_sounds" 
        PRIMARY KEY("item_id","sound_id","sound_type"),

    -- Connect to items table
	CONSTRAINT "fk_items_sounds_item_id" 
        FOREIGN KEY("item_id") REFERENCES "items" ("id") 
        ON DELETE CASCADE ON UPDATE CASCADE,

    -- Connect to sounds table
	CONSTRAINT "fk_items_sounds_sound_id" 
        FOREIGN KEY("sound_id") REFERENCES "sounds" ("id") 
        ON DELETE CASCADE ON UPDATE CASCADE
);