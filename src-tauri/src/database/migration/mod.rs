pub use sea_orm_migration::prelude::*;

mod m20241208_060123_create_items_table;
mod m20241208_060138_create_events_table;
mod m20241208_060144_create_sounds_table;
mod m20241208_060150_create_scripts_table;
mod m20241208_060200_create_commands_table;
mod m20241208_060230_create_model_data_table;
mod m20241208_063859_create_items_impact_sounds_junction_table;
mod m20241210_061820_create_script_events_table;
mod m20241210_082256_create_event_executions_table;
mod m20241210_082316_create_command_executions_table;
mod m20241211_102725_seed_defaults;
mod m20241212_114700_create_key_value_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20241208_060123_create_items_table::Migration),
            Box::new(m20241208_060138_create_events_table::Migration),
            Box::new(m20241208_060144_create_sounds_table::Migration),
            Box::new(m20241208_060150_create_scripts_table::Migration),
            Box::new(m20241208_060200_create_commands_table::Migration),
            Box::new(m20241208_060230_create_model_data_table::Migration),
            Box::new(m20241208_063859_create_items_impact_sounds_junction_table::Migration),
            Box::new(m20241210_061820_create_script_events_table::Migration),
            Box::new(m20241210_082256_create_event_executions_table::Migration),
            Box::new(m20241210_082316_create_command_executions_table::Migration),
            Box::new(m20241211_102725_seed_defaults::Migration),
            Box::new(m20241212_114700_create_key_value_table::Migration),
        ]
    }
}