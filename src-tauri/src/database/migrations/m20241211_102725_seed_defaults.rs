use crate::database::entity::{
    items::{ItemConfig, ItemImageConfig},
    sounds::SoundType,
};
use chrono::Utc;
use itertools::Itertools;
use std::ops::DerefMut;
use uuid::Uuid;

use super::Migration;

pub struct SeedDefaultsMigration;

#[async_trait::async_trait]
impl Migration for SeedDefaultsMigration {
    fn name(&self) -> &str {
        "m20241211_102725_seed_defaults"
    }

    async fn up(&self, db: &crate::database::DbPool) -> anyhow::Result<()> {
        let mut db = db.begin().await?;

        // Populate sounds
        let mut sound_ids: Vec<Uuid> = Vec::new();
        for (name, file_name) in DEFAULT_SOUND_FILES {
            let id = Uuid::new_v4();
            let src = format!("backend://defaults/sounds/{file_name}");
            let volume = 1.0;
            let order = sound_ids.len() as u32;
            let created_at = Utc::now();

            sqlx::query(
                r#"INSERT INTO "sounds" ("id", "name", "src", "volume", "order", "created_at")
                VALUES (?, ?, ?, ?, ?, ?)"#,
            )
            .bind(id)
            .bind(name)
            .bind(src)
            .bind(volume)
            .bind(order)
            .bind(created_at)
            .execute(db.deref_mut())
            .await?;

            sound_ids.push(id);
        }

        // Populate items
        let mut item_ids: Vec<Uuid> = Vec::new();
        for (name, file_name, scale, pixelate) in DEFAULT_THROWABLES {
            let id = Uuid::new_v4();
            let config = ItemConfig {
                image: ItemImageConfig {
                    src: format!("backend://defaults/throwable_images/{file_name}"),
                    pixelate: *pixelate,
                    scale: *scale,
                    weight: 1.0,
                },
                windup: Default::default(),
            };
            let order = item_ids.len() as u32;
            let created_at = Utc::now();
            let config = serde_json::to_value(config)?;

            sqlx::query(
                r#"INSERT INTO "items" ("id", "name", "config", "order", "created_at")
                VALUES (?, ?, ?, ?, ?)"#,
            )
            .bind(id)
            .bind(name)
            .bind(config)
            .bind(order)
            .bind(created_at)
            .execute(db.deref_mut())
            .await?;

            item_ids.push(id);
        }

        // Create item sound associations
        for item_id in item_ids {
            let values_sets = std::iter::repeat("(?,?,?)").take(sound_ids.len()).join(",");
            let sql = format!(
                r#"INSERT INTO "items_sounds" ("item_id", "sound_id", "sound_type")
            VALUES {values_sets}
            "#
            );

            let mut query = sqlx::query(&sql);

            for sound_id in &sound_ids {
                query = query.bind(item_id).bind(sound_id).bind(SoundType::Impact);
            }

            query.execute(db.deref_mut()).await?;
        }

        db.commit().await?;

        Ok(())
    }
}

// Default sound file names
#[rustfmt::skip]
const DEFAULT_SOUND_FILES: &[(&str, &str)] = &[
    ("Seq 2.1 Hit #1 96 HK1", "Seq_2_1_Hit_1_96_HK1.wav"),
    ("Seq 2.1 Hit #2 96 HK1", "Seq_2_1_Hit_2_96_HK1.wav"),
    ("Seq 2.1 Hit #3 96 HK1", "Seq_2_1_Hit_3_96_HK1.wav"),
    ("Seq 2.27 Hit #1 96 HK1", "Seq_2_27_Hit_1_96_HK1.wav"),
    ("Seq 2.27 Hit #2 96 HK1", "Seq_2_27_Hit_2_96_HK1.wav"),
    ("Seq 2.27 Hit #3 96 HK1", "Seq_2_27_Hit_3_96_HK1.wav"),
    ("Seq1.15 Hit #1 96 HK1", "Seq1_15_Hit_1_96_HK1.wav"),
    ("Seq1.15 Hit #2 96 HK1", "Seq1_15_Hit_2_96_HK1.wav"),
    ("Seq1.15 Hit #3 96 HK1", "Seq1_15_Hit_3_96_HK1.wav"),
];

// Default throwable names, scale, pixelation and file names
#[rustfmt::skip]
const DEFAULT_THROWABLES: &[(&str, &str, f32, bool)] = &[
    ("Aluminum Foil", "aluminum_foil.png", 4.0, true),
    ("American Cheese P", "American_cheese_p.png", 4.0, true),    
    ("Bacon", "bacon.png", 4.0, true),
    ("Baking Powder", "baking_powder.png", 4.0, true),
    ("Ball Pen", "ball_pen.png", 4.0, true),
    ("Banana", "banana.png", 4.0, true),
    ("Bandage Box", "bandage_box.png", 4.0, true),
    ("Barbeque Sauce", "barbeque_sauce.png", 4.0, true),
    ("Basket Metal", "basket_metal.png", 4.0, true),
    ("Basket Yellow", "basket_yellow.png", 4.0, true),
    ("Bathroom Cleaner", "bathroom_cleaner.png", 4.0, true),      
    ("Batteries", "batteries.png", 4.0, true),
    ("Bell Pepper", "bell_pepper.png", 4.0, true),
    ("Bell Pepper P", "bell_pepper_p.png", 4.0, true),
    ("Body Lotion", "body_lotion.png", 4.0, true),
    ("Bowl", "bowl.png", 4.0, true),
    ("Bubble Gum", "bubble_gum.png", 4.0, true),
    ("Butter", "butter.png", 4.0, true),
    ("Butter2", "butter2.png", 4.0, true),
    ("Cabbage", "cabbage.png", 4.0, true),
    ("Cabbage P", "cabbage_p.png", 4.0, true),
    ("Candy Bar", "candy_bar.png", 4.0, true),
    ("Cereal1", "cereal1.png", 4.0, true),
    ("Cereal2", "cereal2.png", 4.0, true),
    ("Chopping board", "chopping board.png", 4.0, true),
    ("Cleaning Brush", "cleaning_brush.png", 4.0, true),
    ("Cleaning Gloves", "cleaning_gloves.png", 4.0, true),        
    ("Cleaning Gloves P", "cleaning_gloves_p.png", 4.0, true),    
    ("Coffee Bag", "coffee_bag.png", 4.0, true),
    ("Cookies", "cookies.png", 4.0, true),
    ("Cooking Oil", "cooking_oil.png", 4.0, true),
    ("Credit Card 1", "credit_card_1.png", 4.0, true),
    ("Credit Card 2", "credit_card_2.png", 4.0, true),
    ("Credit Card 3", "credit_card_3.png", 4.0, true),
    ("Detergent", "detergent.png", 4.0, true),
    ("Dry Dog Food", "dry_dog_food.png", 4.0, true),
    ("Egg Box", "egg_box.png", 4.0, true),
    ("Egg Brown", "egg_brown.png", 4.0, true),
    ("Egg Brown P", "egg_brown_p.png", 4.0, true),
    ("Egg White", "egg_white.png", 4.0, true),
    ("Egg White P", "egg_white_p.png", 4.0, true),
    ("Energy Bar", "energy_bar.png", 4.0, true),
    ("Eraser", "eraser.png", 4.0, true),
    ("Eraser P", "eraser_p.png", 4.0, true),
    ("Fish", "fish.png", 4.0, true),
    ("Fish P", "fish_p.png", 4.0, true),
    ("Flour", "flour.png", 4.0, true),
    ("Fruit Cocktail Can", "fruit_cocktail_can.png", 4.0, true),  
    ("Frying pan", "frying pan.png", 4.0, true),
    ("Frying pan P", "frying pan_p.png", 4.0, true),
    ("Glue", "glue.png", 4.0, true),
    ("Glue P", "glue_p.png", 4.0, true),
    ("Glue Stick", "glue_stick.png", 4.0, true),
    ("Grape Soda", "grape_soda.png", 4.0, true),
    ("Green Apple", "green_apple.png", 4.0, true),
    ("Green Apple P", "green_apple_p.png", 4.0, true),
    ("Green Grape", "green_grape.png", 4.0, true),
    ("Green Grape P", "green_grape_p.png", 4.0, true),
    ("Hand Sanitiser", "hand_sanitiser.png", 4.0, true),
    ("Hot Cocoa Mix", "hot_cocoa_mix.png", 4.0, true),
    ("Jam Strawberry", "jam_strawberry.png", 4.0, true),
    ("Ketchup", "ketchup.png", 4.0, true),
    ("Kitchen Knife Set", "kitchen_knife_set.png", 4.0, true),    
    ("Kitchen Knife Set P", "kitchen_knife_set_p.png", 4.0, true),
    ("Kitchen Soap", "kitchen_soap.png", 4.0, true),
    ("Light Bulb", "light_bulb.png", 4.0, true),
    ("Light Bulb Box", "light_bulb_box.png", 4.0, true),
    ("Marshmallows", "marshmallows.png", 4.0, true),
    ("Meat1", "meat1.png", 4.0, true),
    ("Meat1 P", "meat1_p.png", 4.0, true),
    ("Meat2", "meat2.png", 4.0, true),
    ("Meat2 P", "meat2_p.png", 4.0, true),
    ("Milk Bottle", "milk_bottle.png", 4.0, true),
    ("Milk Chocolate", "milk_chocolate.png", 4.0, true),
    ("Milk Gallon", "milk_gallon.png", 4.0, true),
    ("Milk Pack", "milk_pack.png", 4.0, true),
    ("Milk Plastic", "milk_plastic.png", 4.0, true),
    ("Mushroom White", "mushroom_white.png", 4.0, true),
    ("Mushroom White P", "mushroom_white_p.png", 4.0, true),      
    ("Mustard", "mustard.png", 4.0, true),
    ("Olive Oil", "olive_oil.png", 4.0, true),
    ("Orange Juice", "orange_juice.png", 4.0, true),
    ("Paper Bag", "paper_bag.png", 4.0, true),
    ("Peanut Butter", "peanut_butter.png", 4.0, true),
    ("Pencil Box", "pencil_box.png", 4.0, true),
    ("Plain Yogurt", "plain_yogurt.png", 4.0, true),
    ("Potato", "potato.png", 4.0, true),
    ("Potato chip Blue", "potatochip_blue.png", 4.0, true),        
    ("Potato chip Green", "potatochip_green.png", 4.0, true),      
    ("Potato chip Yellow", "potatochip_yellow.png", 4.0, true),    
    ("Potato P", "potato_p.png", 4.0, true),
    ("Power Strip TypeA", "power_strip_typeA.png", 4.0, true),    
    ("Power Strip TypeF", "power_strip_typeF.png", 4.0, true),    
    ("Receipt", "receipt.png", 4.0, true),
    ("Red Apple", "red_apple.png", 4.0, true),
    ("Red Apple P", "red_apple_p.png", 4.0, true),
    ("Red Grape", "red_grape.png", 4.0, true),
    ("Red Grape P", "red_grape_p.png", 4.0, true),
    ("Rolling pin", "rolling pin.png", 4.0, true),
    ("Rubber Duck", "rubber_duck.png", 4.0, true),
    ("Rubber Ducktopus", "rubber_ducktopus.png", 4.0, true),      
    ("Salmon", "salmon.png", 4.0, true),
    ("Salmon P", "salmon_p.png", 4.0, true),
    ("Salt", "salt.png", 4.0, true),
    ("Sausage P", "sausage_p.png", 4.0, true),
    ("Scissors", "scissors.png", 4.0, true),
    ("Scissors P", "scissors_p.png", 4.0, true),
    ("Scrub Brush", "scrub_brush.png", 4.0, true),
    ("Scrub Sponge", "scrub_sponge.png", 4.0, true),
    ("Shampoo", "shampoo.png", 4.0, true),
    ("Sliced Bread P", "sliced_bread_p.png", 4.0, true),
    ("Snack1", "snack1.png", 4.0, true),
    ("Snack2", "snack2.png", 4.0, true),
    ("Soap", "soap.png", 4.0, true),
    ("Soap Box", "soap_box.png", 4.0, true),
    ("Soft Drink Blue", "soft_drink_blue.png", 4.0, true),        
    ("Soft Drink Green", "soft_drink_green.png", 4.0, true),      
    ("Soft Drink Red", "soft_drink_red.png", 4.0, true),
    ("Soft Drink Yellow", "soft_drink_yellow.png", 4.0, true),    
    ("Spatula", "spatula.png", 4.0, true),
    ("Spatula P", "spatula_p.png", 4.0, true),
    ("Strawberry", "strawberry.png", 4.0, true),
    ("Strawberry Ice Cream", "strawberry_ice_cream.png", 4.0, true),
    ("Strawberry Jam", "strawberry_jam.png", 4.0, true),
    ("Strawberry P", "strawberry_p.png", 4.0, true),
    ("Sugar", "sugar.png", 4.0, true),
    ("Sun Cream Tube", "sun_cream_tube.png", 4.0, true),
    ("Teakettle", "teakettle.png", 4.0, true),
    ("Toilet Paper", "toilet_paper.png", 4.0, true),
    ("Toothbrush", "toothbrush.png", 4.0, true),
    ("Toothbrush Set", "toothbrush_set.png", 4.0, true),
    ("Toothpaste", "toothpaste.png", 4.0, true),
    ("Toothpaste Box", "toothpaste_box.png", 4.0, true),
    ("Tuna Can", "tuna_can.png", 4.0, true),
    ("Vanilla Or Lemon Ice Cream", "vanilla_or_lemon_ice_cream.png", 4.0, true),
    ("Water", "water.png", 4.0, true),
    ("Watermelon1", "watermelon1.png", 4.0, true),
    ("Watermelon2", "watermelon2.png", 4.0, true),
    ("Wax", "wax.png", 4.0, true),
    ("Wet Wipe", "wet_wipe.png", 4.0, true),
    ("Whisk", "whisk.png", 4.0, true),
    ("Whisk P", "whisk_p.png", 4.0, true),
    ("White Cheese", "white_cheese.png", 4.0, true),
    ("White Cheese P", "white_cheese_p.png", 4.0, true),
    ("White Cheese Piece", "white_cheese_piece.png", 4.0, true),  
    ("White Cheese Piece P", "white_cheese_piece_p.png", 4.0, true),
    ("Wine Red", "wine_red.png", 4.0, true),
    ("Wine White", "wine_white.png", 4.0, true),
    ("Wine White2", "wine_white2.png", 4.0, true),
    ("Wine White3", "wine_white3.png", 4.0, true),
];
