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
    ("Bacon", "bacon.png", 4.0, true),
    ("Banana", "banana.png", 4.0, true), 
    ("Batteries", "batteries.png", 4.0, true),
    ("Body Lotion", "body_lotion.png", 4.0, true),
    ("Cabbage", "cabbage.png", 4.0, true),
    ("Chopping board", "chopping board.png", 4.0, true),
    ("Credit Card 1", "credit_card_1.png", 4.0, true),
    ("Credit Card 2", "credit_card_2.png", 4.0, true),
    ("Credit Card 3", "credit_card_3.png", 4.0, true),
    ("Egg Brown", "egg_brown.png", 4.0, true),
    ("Egg White", "egg_white.png", 4.0, true),
    ("Fruit Cocktail Can", "fruit_cocktail_can.png", 4.0, true),  
    ("Frying pan", "frying pan.png", 4.0, true),
    ("Glue", "glue.png", 4.0, true),
    ("Glue Stick", "glue_stick.png", 4.0, true),
    ("Grape Soda", "grape_soda.png", 4.0, true),
    ("Green Apple", "green_apple.png", 4.0, true),
    ("Green Grape", "green_grape.png", 4.0, true),
    ("Hand Sanitiser", "hand_sanitiser.png", 4.0, true),
    ("Jam Strawberry", "jam_strawberry.png", 4.0, true),
    ("Ketchup", "ketchup.png", 4.0, true),
    ("Kitchen Soap", "kitchen_soap.png", 4.0, true),
    ("Light Bulb", "light_bulb.png", 4.0, true),
    ("Milk Bottle", "milk_bottle.png", 4.0, true),
    ("Milk Gallon", "milk_gallon.png", 4.0, true),
    ("Milk Pack", "milk_pack.png", 4.0, true),
    ("Milk Plastic", "milk_plastic.png", 4.0, true),
    ("Mustard", "mustard.png", 4.0, true),
    ("Olive Oil", "olive_oil.png", 4.0, true),
    ("Orange Juice", "orange_juice.png", 4.0, true),
    ("Paper Bag", "paper_bag.png", 4.0, true),
    ("Peanut Butter", "peanut_butter.png", 4.0, true),
    ("Plain Yogurt", "plain_yogurt.png", 4.0, true),
    ("Potato", "potato.png", 4.0, true),
    ("Potato chip Blue", "potatochip_blue.png", 4.0, true),        
    ("Potato chip Green", "potatochip_green.png", 4.0, true),      
    ("Potato chip Yellow", "potatochip_yellow.png", 4.0, true),    
    ("Red Apple", "red_apple.png", 4.0, true),
    ("Red Grape", "red_grape.png", 4.0, true),
    ("Rolling pin", "rolling pin.png", 4.0, true),
    ("Rubber Duck", "rubber_duck.png", 4.0, true),
    ("Rubber Ducktopus", "rubber_ducktopus.png", 4.0, true),      
    ("Salt", "salt.png", 4.0, true),
    ("Scissors", "scissors.png", 4.0, true),
    ("Shampoo", "shampoo.png", 4.0, true),
    ("Snack1", "snack1.png", 4.0, true),
    ("Snack2", "snack2.png", 4.0, true),
    ("Soap", "soap.png", 4.0, true),
    ("Soft Drink Blue", "soft_drink_blue.png", 4.0, true),        
    ("Soft Drink Green", "soft_drink_green.png", 4.0, true),      
    ("Soft Drink Red", "soft_drink_red.png", 4.0, true),
    ("Soft Drink Yellow", "soft_drink_yellow.png", 4.0, true),    
    ("Spatula", "spatula.png", 4.0, true),
    ("Strawberry", "strawberry.png", 4.0, true),
    ("Strawberry Ice Cream", "strawberry_ice_cream.png", 4.0, true),
    ("Strawberry Jam", "strawberry_jam.png", 4.0, true),
    ("Sugar", "sugar.png", 4.0, true),
    ("Teakettle", "teakettle.png", 4.0, true),
    ("Toilet Paper", "toilet_paper.png", 4.0, true),
    ("Toothbrush", "toothbrush.png", 4.0, true),
    ("Toothpaste", "toothpaste.png", 4.0, true),
    ("Tuna Can", "tuna_can.png", 4.0, true),
    ("Vanilla Or Lemon Ice Cream", "vanilla_or_lemon_ice_cream.png", 4.0, true),
    ("Water", "water.png", 4.0, true),
    ("Watermelon1", "watermelon1.png", 4.0, true),
    ("Watermelon2", "watermelon2.png", 4.0, true),
    ("Wax", "wax.png", 4.0, true),
    ("Wet Wipe", "wet_wipe.png", 4.0, true),
    ("Whisk", "whisk.png", 4.0, true),
    ("White Cheese", "white_cheese.png", 4.0, true),
    ("White Cheese Piece", "white_cheese_piece.png", 4.0, true),  
    ("Wine Red", "wine_red.png", 4.0, true),
    ("Wine White", "wine_white.png", 4.0, true),
    ("Wine White2", "wine_white2.png", 4.0, true),
    ("Wine White3", "wine_white3.png", 4.0, true),
];
