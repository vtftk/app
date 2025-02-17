use super::{
    m20241208_060123_create_items_table::{ItemsColumn, ItemsTable},
    m20241208_060144_create_sounds_table::{SoundsColumn, SoundsTable},
    Migration,
};
use sea_query::{
    ColumnDef, ForeignKey, ForeignKeyAction, IdenStatic, Index, SqliteQueryBuilder, Table,
};

pub struct ItemsSoundsMigration;

#[async_trait::async_trait]
impl Migration for ItemsSoundsMigration {
    fn name(&self) -> &str {
        "m20241208_063859_create_items_sounds_junction_table"
    }

    async fn up(&self, db: &crate::database::DbPool) -> anyhow::Result<()> {
        sqlx::query(
            &Table::create()
                .table(ItemsSoundsTable)
                .if_not_exists()
                .col(ColumnDef::new(ItemsSoundsColumn::ItemId).uuid().not_null())
                .col(ColumnDef::new(ItemsSoundsColumn::SoundId).uuid().not_null())
                .col(
                    ColumnDef::new(ItemsSoundsColumn::SoundType)
                        .string()
                        .not_null(),
                )
                // Junction table uses a composite key of the item, sound id and sound type combined
                .primary_key(
                    Index::create()
                        .name("pk_items_sounds")
                        .col(ItemsSoundsColumn::ItemId)
                        .col(ItemsSoundsColumn::SoundId)
                        .col(ItemsSoundsColumn::SoundType),
                )
                // Connect to items table
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_items_sounds_item_id")
                        .from(ItemsSoundsTable, ItemsSoundsColumn::ItemId)
                        .to(ItemsTable, ItemsColumn::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade),
                )
                // Connect to sounds table
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_items_sounds_sound_id")
                        .from(ItemsSoundsTable, ItemsSoundsColumn::SoundId)
                        .to(SoundsTable, SoundsColumn::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade),
                )
                .build(SqliteQueryBuilder),
        )
        .execute(db)
        .await?;

        Ok(())
    }
}

#[derive(IdenStatic, Copy, Clone)]
#[iden(rename = "items_sounds")]
pub struct ItemsSoundsTable;

#[derive(IdenStatic, Copy, Clone)]
pub enum ItemsSoundsColumn {
    ItemId,
    SoundId,
    SoundType,
}
