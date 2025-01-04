use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TwitchAccess::Table)
                    .if_not_exists()
                    .col(pk_auto(TwitchAccess::Id))
                    .col(string(TwitchAccess::AccessToken))
                    .col(json(TwitchAccess::Scopes))
                    .col(date_time(TwitchAccess::CreatedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TwitchAccess::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum TwitchAccess {
    Table,
    Id,
    AccessToken,
    Scopes,
    CreatedAt,
}
