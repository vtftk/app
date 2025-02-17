use chrono::{DateTime, Utc};
use sea_query::{Expr, IdenStatic, OnConflict, Query};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::database::{
    helpers::{sql_exec, sql_query_maybe_one},
    DbPool, DbResult,
};

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

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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
    pub async fn set(db: &DbPool, create: SetSecret) -> DbResult<()> {
        let created_at = Utc::now();

        sql_exec(
            db,
            Query::insert()
                .into_table(SecretsTable)
                .columns([
                    SecretsColumn::Key,
                    SecretsColumn::Value,
                    SecretsColumn::Metadata,
                    SecretsColumn::CreatedAt,
                ])
                .values_panic([
                    create.key.into(),
                    create.value.into(),
                    create.metadata.into(),
                    created_at.into(),
                ])
                .on_conflict(
                    OnConflict::column(SecretsColumn::Key)
                        .update_columns([
                            SecretsColumn::Value,
                            SecretsColumn::Metadata,
                            SecretsColumn::CreatedAt,
                        ])
                        .to_owned(),
                ),
        )
        .await
    }

    /// Find a specific key value by key
    pub async fn get_by_key(db: &DbPool, key: &str) -> DbResult<Option<Self>> {
        sql_query_maybe_one(
            db,
            Query::select()
                .from(SecretsTable)
                .columns([
                    SecretsColumn::Key,
                    SecretsColumn::Value,
                    SecretsColumn::Metadata,
                    SecretsColumn::CreatedAt,
                ])
                .and_where(Expr::col(SecretsColumn::Key).eq(key)),
        )
        .await
    }

    /// Find a specific key value by key
    pub async fn delete_by_key(db: &DbPool, key: &str) -> DbResult<()> {
        sql_exec(
            db,
            Query::delete()
                .from_table(SecretsTable)
                .and_where(Expr::col(SecretsColumn::Key).eq(key)),
        )
        .await
    }
}
