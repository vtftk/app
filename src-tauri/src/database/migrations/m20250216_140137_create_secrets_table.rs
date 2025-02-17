use super::Migration;
use sea_query::{ColumnDef, IdenStatic, SqliteQueryBuilder, Table};

pub struct SecretsMigration;

#[async_trait::async_trait]
impl Migration for SecretsMigration {
    fn name(&self) -> &str {
        "m20250216_140137_create_secrets_table"
    }

    async fn up(&self, db: &crate::database::DbPool) -> anyhow::Result<()> {
        sqlx::query(
            &Table::create()
                .table(SecretsTable)
                .if_not_exists()
                .col(
                    ColumnDef::new(SecretsColumn::Key)
                        .string()
                        .not_null()
                        .primary_key(),
                )
                .col(ColumnDef::new(SecretsColumn::Value).string().not_null())
                .col(
                    ColumnDef::new(SecretsColumn::Metadata)
                        .json_binary()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(SecretsColumn::CreatedAt)
                        .date_time()
                        .not_null(),
                )
                .build(SqliteQueryBuilder),
        )
        .execute(db)
        .await?;

        Ok(())
    }
}

#[derive(IdenStatic, Copy, Clone)]
#[iden(rename = "secrets")]
pub struct SecretsTable;

#[derive(IdenStatic, Copy, Clone)]
pub enum SecretsColumn {
    /// Unique key the secret is stored under
    Key,
    /// Value of the secret
    Value,
    /// Additional metadata stored with the secret
    Metadata,
    CreatedAt,
}
