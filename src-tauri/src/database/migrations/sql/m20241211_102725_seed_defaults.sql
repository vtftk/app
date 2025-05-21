BEGIN TRANSACTION;

-- Create a temporary table to store the sounds we will be using
-- (They contain the incomplete sound data needed to a fill a query)
CREATE TEMP TABLE "sounds_temp" (
	"id"	uuid_text NOT NULL PRIMARY KEY,
	"name"	VARCHAR NOT NULL,
	"src"	VARCHAR NOT NULL
);


-- Create a temporary table to store the items we will be using
-- (They contain the incomplete item data needed to a fill a query)
CREATE TEMP TABLE "items_temp" (
	"id"	uuid_text NOT NULL PRIMARY KEY,
	"name"	VARCHAR NOT NULL,
	"src"	VARCHAR NOT NULL,
	"scale"	FLOAT NOT NULL,
	"pixelate"	BOOLEAN NOT NULL
);

-- Populate the temp sounds table
INSERT INTO "sounds_temp" ("id", "name", "src") VALUES 
(X'd15cd471d25e4513b2004674982e8c4f', 'Seq 2.1 Hit #1 96 HK1', 'Seq_2_1_Hit_1_96_HK1.wav'),
(X'e26190c98b0449cc9df8d85f26ac6dc9', 'Seq 2.1 Hit #2 96 HK1', 'Seq_2_1_Hit_2_96_HK1.wav'),
(X'866dd6694fd94da29ed21147f62fb151', 'Seq 2.1 Hit #3 96 HK1', 'Seq_2_1_Hit_3_96_HK1.wav'),
(X'07171182499d4afaa65e5d97077f33f0', 'Seq 2.27 Hit #1 96 HK1', 'Seq_2_27_Hit_1_96_HK1.wav'),
(X'f7eef2f9c51d4adf8f9d0a6a8aadbcf9', 'Seq 2.27 Hit #2 96 HK1', 'Seq_2_27_Hit_2_96_HK1.wav'),
(X'3e1869619926485090009ed8b6c3bca9', 'Seq 2.27 Hit #3 96 HK1', 'Seq_2_27_Hit_3_96_HK1.wav'),
(X'b418c86fd2e24392af8ae1e9b81a9779', 'Seq1.15 Hit #1 96 HK1', 'Seq1_15_Hit_1_96_HK1.wav'),
(X'f9a5cb23bb8c4d6a95ddc2383dc8c6df', 'Seq1.15 Hit #2 96 HK1', 'Seq1_15_Hit_2_96_HK1.wav'),
(X'b9ebd5cb99d94863a0fae17425c5a998', 'Seq1.15 Hit #3 96 HK1', 'Seq1_15_Hit_3_96_HK1.wav');


-- Populate the temp items table
INSERT INTO "items_temp" ("id", "name", "src", "scale", "pixelate") VALUES 
(X'73357e1a3f814a6aaf7fbdf63936d03a', 'Bacon', 'bacon.png', 4.0,TRUE),
(X'192a73d8ae174cdc852bf8362f927500', 'Banana', 'banana.png', 4.0,TRUE),
(X'c6e934fd98804afe865c37eaccbfbea6', 'Batteries', 'batteries.png', 4.0,TRUE),
(X'63c041c9d9654a4cb7c339103a6c02dc', 'Body Lotion', 'body_lotion.png', 4.0,TRUE),
(X'dc928197507d459db3d8bf47ff51dd88', 'Cabbage', 'cabbage.png', 4.0,TRUE),
(X'00096b6b13d9400290e609ac0116ba14', 'Chopping board', 'chopping board.png', 4.0,TRUE),
(X'6a404494c02f41199448ef13513d1fd5', 'Credit Card 1', 'credit_card_1.png', 4.0,TRUE),
(X'4ca7530f747a49a89707128e821b4d42', 'Credit Card 2', 'credit_card_2.png', 4.0,TRUE),
(X'39ba66422c46488cbaba6f736049b553', 'Credit Card 3', 'credit_card_3.png', 4.0,TRUE),
(X'98ee76c934e84b478faf7b5b633bf33c', 'Egg Brown', 'egg_brown.png', 4.0,TRUE),
(X'bc655cd4f34946c88b9f0fac780a22ce', 'Egg White', 'egg_white.png', 4.0,TRUE),
(X'2b181cc97adf4b5789a78f8e735508db', 'Fruit Cocktail Can', 'fruit_cocktail_can.png', 4.0,TRUE),
(X'3e8344d28096402c85bd7fbf1be3f90b', 'Frying pan', 'frying pan.png', 4.0,TRUE),
(X'7254f1f77dda43068ebcb73d7f50d5af', 'Glue', 'glue.png', 4.0,TRUE),
(X'b3c5b6bd0a6b4c6f99fb5b9947d06441', 'Glue Stick', 'glue_stick.png', 4.0,TRUE),
(X'3eafad96e08b4a8096947c4bd3c1c981', 'Grape Soda', 'grape_soda.png', 4.0,TRUE),
(X'f756d8f186604e0d84f20bcaeed12a7b', 'Green Apple', 'green_apple.png', 4.0,TRUE),
(X'7af7de6daeb748e79525ed3b5863c3a3', 'Green Grape', 'green_grape.png', 4.0,TRUE),
(X'3f28606846a8440084eca8cd49739cba', 'Hand Sanitiser', 'hand_sanitiser.png', 4.0,TRUE),
(X'778fac629c574b23aa478f15623c9f4e', 'Jam Strawberry', 'jam_strawberry.png', 4.0,TRUE),
(X'1e389ba3fb684bf3a66e7bb32407874e', 'Ketchup', 'ketchup.png', 4.0,TRUE),
(X'086357e24bbe48809946e9236c563445', 'Kitchen Soap', 'kitchen_soap.png', 4.0,TRUE),
(X'1310cc160bd54b50a726c6dc6e957443', 'Light Bulb', 'light_bulb.png', 4.0,TRUE),
(X'0bc80a60457c4ec39f8d525c73c23c30', 'Milk Bottle', 'milk_bottle.png', 4.0,TRUE),
(X'2272730587dc4e749c11b7a46548d17d', 'Milk Gallon', 'milk_gallon.png', 4.0,TRUE),
(X'48e8d861e9e340ea9346e2c19924bba3', 'Milk Pack', 'milk_pack.png', 4.0,TRUE),
(X'5a37affc52fd4a98b323849f75cb2648', 'Milk Plastic', 'milk_plastic.png', 4.0,TRUE),
(X'd8fc2fbdac5a4caa912018fae62bb4a4', 'Mustard', 'mustard.png', 4.0,TRUE),
(X'af7479d2e4b5415eb65e8f4297e38891', 'Olive Oil', 'olive_oil.png', 4.0,TRUE),
(X'bc565a467e8b43b085d6430e604a3ff4', 'Orange Juice', 'orange_juice.png', 4.0,TRUE),
(X'2bef164569454902910e1f77256a1baf', 'Paper Bag', 'paper_bag.png', 4.0,TRUE),
(X'1c1ce961ba04427c9accf3ca52ea58cb', 'Peanut Butter', 'peanut_butter.png', 4.0,TRUE),
(X'98eb262babd74cfa9448dd626ed4cb41', 'Plain Yogurt', 'plain_yogurt.png', 4.0,TRUE),
(X'd62f3201a4de401ba5e00cd83a81771e', 'Potato', 'potato.png', 4.0,TRUE),
(X'd556638815d34a3ba4a3922230c4af93', 'Potato chip Blue', 'potatochip_blue.png', 4.0,TRUE),
(X'5ebe643c6bb74fc0b3ef8abdb3bf6621', 'Potato chip Green', 'potatochip_green.png', 4.0,TRUE),
(X'7ee67a5160e64ee586e732743df98223', 'Potato chip Yellow', 'potatochip_yellow.png', 4.0,TRUE),
(X'8821e6eebf3c4e5085b4ea5429cb210d', 'Red Apple', 'red_apple.png', 4.0,TRUE),
(X'37fc638d161947738f26f046068bb0ac', 'Red Grape', 'red_grape.png', 4.0,TRUE),
(X'60873c88967c4428b3129ce6e99c7397', 'Rolling pin', 'rolling pin.png', 4.0,TRUE),
(X'd337fcc7e86a45d4944b9affedae6d20', 'Rubber Duck', 'rubber_duck.png', 4.0,TRUE),
(X'8e2fe87bdae04836becdcd5ca360edf9', 'Rubber Ducktopus', 'rubber_ducktopus.png', 4.0,TRUE),
(X'8e34de6b8e71440d9250c95d1a74b1cc', 'Salt', 'salt.png', 4.0,TRUE),
(X'6be8f19b5b6a4d04aa9ffd295933c46d', 'Scissors', 'scissors.png', 4.0,TRUE),
(X'b2e86848ed9c4eeabc63ad7da8cbc6fa', 'Shampoo', 'shampoo.png', 4.0,TRUE),
(X'a01e9719f24e4d6a8e8bc8c87c422dcf', 'Snack1', 'snack1.png', 4.0,TRUE),
(X'7a851ca7066d4c228c60e43235a27e8c', 'Snack2', 'snack2.png', 4.0,TRUE),
(X'6e593cabb25746eca14aca4a155a8bd0', 'Soap', 'soap.png', 4.0,TRUE),
(X'd90dcb831d7441a29f56c5ddac6f6f35', 'Soft Drink Blue', 'soft_drink_blue.png', 4.0,TRUE),
(X'0b3335412dcc43f5b73966c4c8e5413e', 'Soft Drink Green', 'soft_drink_green.png', 4.0,TRUE),
(X'609e5ba894144de29eacf07a2485850d', 'Soft Drink Red', 'soft_drink_red.png', 4.0,TRUE),
(X'2f238f27176b487d82566c7c9ceeb3bf', 'Soft Drink Yellow', 'soft_drink_yellow.png', 4.0,TRUE),
(X'bc7a0e5fa9b34821927a592ea391bf39', 'Spatula', 'spatula.png', 4.0,TRUE),
(X'e5af691083de4951b05bd821c3ca9692', 'Strawberry', 'strawberry.png', 4.0,TRUE),
(X'f6a70660621b42f39990c1c43783ce01', 'Strawberry Ice Cream', 'strawberry_ice_cream.png', 4.0,TRUE),
(X'fc05215f6219494ba592cf7552b2b0b4', 'Strawberry Jam', 'strawberry_jam.png', 4.0,TRUE),
(X'eb7cf94abece43d3b82b50e3b805dba6', 'Sugar', 'sugar.png', 4.0,TRUE),
(X'712c3621906e4332a89d330b3b52940c', 'Teakettle', 'teakettle.png', 4.0,TRUE),
(X'4e43a3ae74ef4c4fabe0fb6682a5dc24', 'Toilet Paper', 'toilet_paper.png', 4.0,TRUE),
(X'96ebd291e43d491990a5d427f54f0ff0', 'Toothbrush', 'toothbrush.png', 4.0,TRUE),
(X'98b42c21c84e40bbb174286b79d4c665', 'Toothpaste', 'toothpaste.png', 4.0,TRUE),
(X'35a6c6a4cc354eff9a85898742706b41', 'Tuna Can', 'tuna_can.png', 4.0,TRUE),
(X'ec7e323493174fd3a367378f26ebea7c', 'Vanilla Or Lemon Ice Cream', 'vanilla_or_lemon_ice_cream.png', 4.0,TRUE),   
(X'53988a4d3e924b63b9ee7406e46fa38e', 'Water', 'water.png', 4.0,TRUE),
(X'c72f9c63b9064104af1dfa15f2533f6d', 'Watermelon1', 'watermelon1.png', 4.0,TRUE),
(X'9bd3a9e9582e4844a17c159b884e93a9', 'Watermelon2', 'watermelon2.png', 4.0,TRUE),
(X'955b6e89d3b249169eb2c9ea4a84449e', 'Wax', 'wax.png', 4.0,TRUE),
(X'482468ad882d4be68157b714e40ee721', 'Wet Wipe', 'wet_wipe.png', 4.0,TRUE),
(X'10e35bf4e1cb4eadb01635e7d46d625c', 'Whisk', 'whisk.png', 4.0,TRUE),
(X'628de799477d4cfea12f6e9136044bfa', 'White Cheese', 'white_cheese.png', 4.0,TRUE),
(X'86a4c05d09bf43cd9c367bfe58b725e6', 'White Cheese Piece', 'white_cheese_piece.png', 4.0,TRUE),
(X'97a92f270c134dc493524d1e23ac8c49', 'Wine Red', 'wine_red.png', 4.0,TRUE),
(X'0cabf7284e2a4e69b87cd10dcd59d3dc', 'Wine White', 'wine_white.png', 4.0,TRUE),
(X'dc7408ada6be46f3a6c112539ca5a456', 'Wine White2', 'wine_white2.png', 4.0,TRUE),
(X'abd8d7cc531d454783d327833f3d58b6', 'Wine White3', 'wine_white3.png', 4.0,TRUE);

-- Use the temp sounds table to populate the actual sounds table
INSERT INTO "sounds" ("id", "name", "src", "volume", "order", "created_at")
SELECT 
    "temp"."id",
    "temp"."name",
    -- Join the default backend URL onto the SRC
    'backend://defaults/sounds/' || "temp"."src",
    1.0,
    -- Get highest order + 1 from sounds table
    (SELECT IFNULL(MAX("order"), 0) + 1 FROM "sounds"),
    -- Hard coded timestamp
    '2024-12-12T12:51:28.720892300+00:00'
FROM "sounds_temp" "temp";

-- Create the items from the temp
INSERT INTO "items"  ("id", "name", "config", "order", "created_at")
SELECT 
    "temp"."id",
    "temp"."name",
    json_object(
        'image', json_object(
            'src', 'backend://defaults/throwable_images/' || "temp"."src",
            'pixelate', CASE WHEN "temp"."pixelate" THEN json('true') ELSE json('false') END,
            'scale', "temp"."scale",
            'weight', 1.0
        )
    ),
    (SELECT IFNULL(MAX("order"), 0) + 1 FROM "items"),
    '2024-12-12T12:51:28.720892300+00:00'
FROM "items_temp" "temp";

-- Relate all the added sounds to all the added items
INSERT INTO "items_sounds" ("item_id", "sound_id", "sound_type")
SELECT "item"."id", "sound"."id", 'Impact'
FROM "items_temp" "item"
CROSS JOIN "sounds_temp" "sound";

-- Drop temporary tables
DROP TABLE "items_temp";
DROP TABLE "sounds_temp";

COMMIT;