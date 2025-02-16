use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Secrets::Table)
                    .if_not_exists()
                    .col(string(Secrets::Key).primary_key().to_owned())
                    .col(string(Secrets::Value))
                    .col(json(Secrets::Metadata))
                    .col(date_time(Secrets::CreatedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Secrets::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Secrets {
    Table,
    /// Unique key the secret is stored under
    Key,
    /// Value of the secret
    Value,
    /// Additional metadata stored with the secret
    Metadata,
    CreatedAt,
}
