use sea_query::{Expr, IdenStatic, OnConflict, Query, Value};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use strum::{Display, EnumString};

use crate::database::{
    helpers::{sql_exec, sql_query_maybe_one},
    DbPool, DbResult,
};

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct KeyValueModel {
    /// Key for the key value pair
    pub key: String,
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub ty: KeyValueType,
    pub value: String,
}

/// Key value type
#[derive(Debug, Copy, Clone, Serialize, Deserialize, EnumString, Display, sqlx::Type)]
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

impl From<KeyValueType> for Value {
    fn from(x: KeyValueType) -> Value {
        let string: String = x.to_string();
        Value::String(Some(Box::new(string)))
    }
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
    pub async fn create(db: &DbPool, create: CreateKeyValue) -> DbResult<()> {
        sql_exec(
            db,
            Query::insert()
                .into_table(KeyValueTable)
                .columns([
                    KeyValueColumn::Key,
                    KeyValueColumn::Value,
                    KeyValueColumn::Type,
                ])
                .values_panic([create.key.into(), create.value.into(), create.ty.into()])
                .on_conflict(
                    OnConflict::column(KeyValueColumn::Key)
                        .update_columns([KeyValueColumn::Value, KeyValueColumn::Type])
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
                .from(KeyValueTable)
                .columns([
                    KeyValueColumn::Key,
                    KeyValueColumn::Value,
                    KeyValueColumn::Type,
                ])
                .and_where(Expr::col(KeyValueColumn::Key).eq(key)),
        )
        .await
    }

    /// Find a specific key value by key
    pub async fn delete_by_key(db: &DbPool, key: &str) -> DbResult<()> {
        sql_exec(
            db,
            Query::delete()
                .from_table(KeyValueTable)
                .and_where(Expr::col(KeyValueColumn::Key).eq(key)),
        )
        .await
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
