use anyhow::Context;
use sea_orm::{entity::prelude::*, ActiveValue::Set, FromJsonQueryResult};
use serde::{Deserialize, Serialize};

use super::shared::DbResult;

// Type alias helpers for the database entity types
pub type CommandExecutionModel = Model;
pub type CommandExecutionEntity = Entity;
pub type CommandExecutionActiveModel = ActiveModel;
pub type CommandExecutionColumn = Column;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "command_executions")]
pub struct Model {
    /// Unique ID for the event
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub command_id: Uuid,
    pub metadata: CommandExecutionMetadata,
    pub created_at: DateTimeUtc,
}

#[derive(Clone, Debug, PartialEq, FromJsonQueryResult, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CommandExecutionMetadata(pub serde_json::Value);

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /// Relationship to the command
    #[sea_orm(
        belongs_to = "super::commands::Entity",
        from = "Column::CommandId",
        to = "super::commands::Column::Id"
    )]
    Command,
}

impl Related<super::commands::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Command.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug)]
pub struct CreateCommandExecution {
    pub command_id: Uuid,
    pub metadata: serde_json::Value,
    pub created_at: DateTimeUtc,
}

impl Model {
    /// Create a new script
    pub async fn create<C>(db: &C, create: CreateCommandExecution) -> anyhow::Result<Model>
    where
        C: ConnectionTrait + Send + 'static,
    {
        let id = Uuid::new_v4();
        let active_model = ActiveModel {
            id: Set(id),
            command_id: Set(create.command_id),
            metadata: Set(CommandExecutionMetadata(create.metadata)),
            created_at: Set(create.created_at),
        };

        Entity::insert(active_model)
            .exec_without_returning(db)
            .await?;

        let model = Self::get_by_id(db, id)
            .await?
            .context("model was not inserted")?;

        Ok(model)
    }

    pub async fn get_by_id<C>(db: &C, id: Uuid) -> DbResult<Option<Self>>
    where
        C: ConnectionTrait + Send + 'static,
    {
        Entity::find_by_id(id).one(db).await
    }

    /// Unused, may be used in future to display previous executions in the UI
    #[allow(unused)]
    pub async fn all<C>(db: &C) -> DbResult<Vec<Self>>
    where
        C: ConnectionTrait + Send + 'static,
    {
        Entity::find().all(db).await
    }
}