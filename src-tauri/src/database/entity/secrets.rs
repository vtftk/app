use chrono::{DateTime, Utc};
use sea_query::{Expr, IdenStatic, OnConflict, Query, SqliteQueryBuilder};
use sea_query_binder::SqlxBinder;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::database::{DbPool, DbResult};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, FromRow)]
pub struct SecretsModel {
    pub key: String,
    pub value: String,
    #[sqlx(jsn)]
    pub metadata: serde_json::Value,
    // Date time of creation
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SetSecret {
    pub key: String,
    pub value: String,
    pub metadata: serde_json::Value,
}

impl SecretsModel {
    /// Create a new sound
    pub async fn set(db: &DbPool, create: SetSecret) -> anyhow::Result<SecretsModel> {
        let model = SecretsModel {
            key: create.key,
            value: create.value,
            metadata: create.metadata,
            created_at: Utc::now(),
        };

        let (sql, value) = Query::insert()
            .into_table(SecretsTable)
            .columns([
                SecretsColumn::Key,
                SecretsColumn::Value,
                SecretsColumn::Metadata,
                SecretsColumn::CreatedAt,
            ])
            .values_panic([
                model.key.clone().into(),
                model.value.clone().into(),
                model.metadata.clone().into(),
                model.created_at.into(),
            ])
            .on_conflict(
                OnConflict::column(SecretsColumn::Key)
                    .update_columns([
                        SecretsColumn::Value,
                        SecretsColumn::Metadata,
                        SecretsColumn::CreatedAt,
                    ])
                    .to_owned(),
            )
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, value).execute(db).await?;

        Ok(model)
    }

    /// Find a specific key value by key
    pub async fn get_by_key(db: &DbPool, key: &str) -> DbResult<Option<Self>> {
        let (sql, values) = Query::select()
            .from(SecretsTable)
            .columns([
                SecretsColumn::Key,
                SecretsColumn::Value,
                SecretsColumn::Metadata,
                SecretsColumn::CreatedAt,
            ])
            .and_where(Expr::col(SecretsColumn::Key).eq(key))
            .build_sqlx(SqliteQueryBuilder);

        let result = sqlx::query_as_with(&sql, values).fetch_optional(db).await?;
        Ok(result)
    }

    /// Find a specific key value by key
    pub async fn delete_by_key(db: &DbPool, key: &str) -> DbResult<()> {
        let (sql, values) = Query::delete()
            .from_table(SecretsTable)
            .and_where(Expr::col(SecretsColumn::Key).eq(key))
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(db).await?;
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
