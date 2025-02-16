use super::shared::DbResult;
use anyhow::Context;
use chrono::Utc;
use sea_orm::{entity::prelude::*, sea_query::OnConflict, ActiveValue::Set, FromJsonQueryResult};
use serde::{Deserialize, Serialize};

// Type alias helpers for the database entity types
pub type SecretModel = Model;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "secrets")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub key: String,
    pub value: String,
    pub metadata: SecretMetadata,
    // Date time of creation
    pub created_at: DateTimeUtc,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(transparent)]
pub struct SecretMetadata(pub serde_json::Value);

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Deserialize)]
pub struct SetSecret {
    pub key: String,
    pub value: String,
    pub metadata: serde_json::Value,
}

impl Model {
    /// Create a new sound
    pub async fn set<C>(db: &C, create: SetSecret) -> anyhow::Result<Model>
    where
        C: ConnectionTrait + Send + 'static,
    {
        let active_model = ActiveModel {
            key: Set(create.key.to_string()),
            value: Set(create.value),
            metadata: Set(SecretMetadata(create.metadata)),
            created_at: Set(Utc::now()),
        };

        Entity::insert(active_model)
            .on_conflict(
                OnConflict::column(Column::Key)
                    .update_columns([Column::Value, Column::Metadata, Column::CreatedAt])
                    .to_owned(),
            )
            .exec_without_returning(db)
            .await?;

        let model = Self::get(db, &create.key)
            .await?
            .context("model was not inserted")?;

        Ok(model)
    }

    pub async fn get<C>(db: &C, key: &str) -> DbResult<Option<Self>>
    where
        C: ConnectionTrait + Send + 'static,
    {
        Entity::find_by_id(key).one(db).await
    }

    pub async fn delete<C>(db: &C, key: &str) -> DbResult<()>
    where
        C: ConnectionTrait + Send + 'static,
    {
        Entity::delete_by_id(key).exec(db).await?;
        Ok(())
    }
}
