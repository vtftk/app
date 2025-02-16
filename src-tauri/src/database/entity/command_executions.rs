use anyhow::Context;
use sea_orm::{
    entity::prelude::*, sea_query::Func, ActiveValue::Set, FromJsonQueryResult, FromQueryResult,
    QuerySelect,
};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::events::TwitchEventUser;

use super::shared::DbResult;

// Type alias helpers for the database entity types
pub type CommandExecutionModel = Model;
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

#[serde_as]
#[derive(Clone, Debug, PartialEq, FromJsonQueryResult, Serialize, Deserialize)]
pub struct CommandExecutionMetadata {
    /// User who triggered the event
    pub user: Option<TwitchEventUser>,

    /// Catchall for any other metadata
    #[serde(flatten)]
    #[serde_as(as = "serde_with::Map<_, _>")]
    pub data: Vec<(String, serde_json::Value)>,
}

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
    pub metadata: CommandExecutionMetadata,
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
            metadata: Set(create.metadata),
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

    pub async fn delete_before<C>(db: &C, start_date: DateTimeUtc) -> DbResult<()>
    where
        C: ConnectionTrait + Send + 'static,
    {
        Entity::delete_many()
            .filter(Column::CreatedAt.lt(start_date))
            .exec(db)
            .await?;
        Ok(())
    }

    pub async fn delete_many<C>(db: &C, ids: &[Uuid]) -> DbResult<()>
    where
        C: ConnectionTrait + Send + 'static,
    {
        Entity::delete_many()
            .filter(Column::Id.is_in(ids.iter().copied()))
            .exec(db)
            .await?;
        Ok(())
    }

    pub async fn estimate_size<C>(db: &C) -> DbResult<u32>
    where
        C: ConnectionTrait + Send + 'static,
    {
        #[derive(Default, FromQueryResult)]
        struct PartialModel {
            total_message_length: Option<u32>,
        }

        let result = Entity::find()
            .expr_as(
                Func::sum(Func::char_length(Expr::col(Column::Metadata))),
                "total_message_length",
            )
            .into_model::<PartialModel>()
            .one(db)
            .await?;

        let result = match result {
            Some(result) => result,
            None => return Ok(0),
        };

        Ok(result.total_message_length.unwrap_or_default())
    }
}
