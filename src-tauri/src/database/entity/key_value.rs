use sea_query::{Expr, IdenStatic, OnConflict, Query, SqliteQueryBuilder};
use sea_query_binder::SqlxBinder;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use strum::{Display, EnumString};

use crate::database::{DbPool, DbResult};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, FromRow)]
pub struct KeyValueModel {
    /// Key for the key value pair
    pub key: String,
    #[serde(rename = "type")]
    pub ty: KeyValueType,
    pub value: String,
}

/// Key value type
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, EnumString, Display, sqlx::Type,
)]
pub enum KeyValueType {
    /// Plain text is stored
    Text,
    /// Number is stored as plain text
    Number,
    /// Object is stored as JSON
    Object,
    /// Array is stored as JSON
    Array,
}

#[derive(Debug, Deserialize)]
pub struct CreateKeyValue {
    pub key: String,
    pub value: String,
    #[serde(rename = "type")]
    pub ty: KeyValueType,
}

impl KeyValueModel {
    /// Create a new sound
    pub async fn create(db: &DbPool, create: CreateKeyValue) -> anyhow::Result<KeyValueModel> {
        let model = KeyValueModel {
            key: create.key.to_string(),
            value: create.value.to_string(),
            ty: create.ty,
        };

        let (sql, value) = Query::insert()
            .into_table(KeyValueTable)
            .columns([
                KeyValueColumn::Key,
                KeyValueColumn::Value,
                KeyValueColumn::Type,
            ])
            .values_panic([
                create.key.into(),
                create.value.into(),
                create.ty.to_string().into(),
            ])
            .on_conflict(
                OnConflict::column(KeyValueColumn::Key)
                    .update_columns([KeyValueColumn::Value, KeyValueColumn::Type])
                    .to_owned(),
            )
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, value).execute(db).await?;

        Ok(model)
    }

    /// Find a specific key value by key
    pub async fn get_by_key(db: &DbPool, key: &str) -> DbResult<Option<Self>> {
        let (sql, values) = Query::select()
            .from(KeyValueTable)
            .columns([
                KeyValueColumn::Key,
                KeyValueColumn::Value,
                KeyValueColumn::Type,
            ])
            .and_where(Expr::col(KeyValueColumn::Key).eq(key))
            .build_sqlx(SqliteQueryBuilder);

        let result = sqlx::query_as_with(&sql, values).fetch_optional(db).await?;
        Ok(result)
    }

    /// Find a specific key value by key
    pub async fn delete_by_key(db: &DbPool, key: &str) -> DbResult<()> {
        let (sql, values) = Query::delete()
            .from_table(KeyValueTable)
            .and_where(Expr::col(KeyValueColumn::Key).eq(key))
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(db).await?;
        Ok(())
    }
}

#[derive(IdenStatic, Copy, Clone)]
#[iden(rename = "key_value")]
pub struct KeyValueTable;

#[derive(IdenStatic, Copy, Clone)]
pub enum KeyValueColumn {
    Key,
    Value,
    Type,
}
