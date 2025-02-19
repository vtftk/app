use chrono::{DateTime, Utc};
use sea_query::{CaseStatement, Expr, Func, IdenStatic, Order, Query, Value};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::database::{
    helpers::{sql_exec, sql_query_all, sql_query_maybe_one},
    DbPool, DbResult,
};

use super::shared::UpdateOrdering;

#[derive(IdenStatic, Copy, Clone)]
#[iden(rename = "sounds")]
pub struct SoundsTable;

#[derive(IdenStatic, Copy, Clone)]
pub enum SoundsColumn {
    Id,
    Name,
    Src,
    Volume,
    Order,
    CreatedAt,
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, strum::Display, strum::EnumString, sqlx::Type,
)]
pub enum SoundType {
    Impact,
    Windup,
}

impl From<SoundType> for Value {
    fn from(x: SoundType) -> Value {
        let string: String = x.to_string();
        Value::String(Some(Box::new(string)))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SoundModel {
    /// Unique ID for the sound
    pub id: Uuid,
    /// Name of the sound
    pub name: String,
    /// Src URL for the image
    pub src: String,
    /// Volume of the sound 0-1
    pub volume: f32,
    /// Ordering
    pub order: u32,
    // Date time of creation
    pub created_at: DateTime<Utc>,
}

/// Partial chunk of the sound model used for compute
/// purposes, excludes fields used by the UI
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PartialSoundModel {
    /// Unique ID for the sound
    pub id: Uuid,
    /// Src URL for the image
    pub src: String,
    /// Volume of the sound 0-1
    pub volume: f32,
}

#[derive(Debug, Deserialize)]
pub struct CreateSound {
    pub name: String,
    pub src: String,
    pub volume: f32,
}

#[derive(Default, Deserialize)]
pub struct UpdateSound {
    pub name: Option<String>,
    pub src: Option<String>,
    pub volume: Option<f32>,
    pub order: Option<u32>,
}

impl SoundModel {
    fn columns() -> [SoundsColumn; 6] {
        [
            SoundsColumn::Id,
            SoundsColumn::Name,
            SoundsColumn::Src,
            SoundsColumn::Volume,
            SoundsColumn::Order,
            SoundsColumn::CreatedAt,
        ]
    }

    /// Create a sound
    pub async fn create(db: &DbPool, create: CreateSound) -> anyhow::Result<SoundModel> {
        let id = Uuid::new_v4();
        let model = SoundModel {
            id,
            name: create.name,
            src: create.src,
            volume: create.volume,
            order: 0,
            created_at: Utc::now(),
        };

        sql_exec(
            db,
            Query::insert()
                .into_table(SoundsTable)
                .columns(SoundModel::columns())
                .values_panic([
                    model.id.into(),
                    model.name.clone().into(),
                    model.src.clone().into(),
                    model.volume.into(),
                    model.order.into(),
                    model.created_at.into(),
                ]),
        )
        .await?;

        Ok(model)
    }

    /// Find a specific sound by ID
    pub async fn get_by_id(db: &DbPool, id: Uuid) -> DbResult<Option<SoundModel>> {
        sql_query_maybe_one(
            db,
            Query::select()
                .columns(SoundModel::columns())
                .from(SoundsTable)
                .and_where(Expr::col(SoundsColumn::Id).eq(id)),
        )
        .await
    }

    /// Find a specific sound by ID
    pub async fn get_by_id_partial(db: &DbPool, id: Uuid) -> DbResult<Option<PartialSoundModel>> {
        sql_query_maybe_one(
            db,
            Query::select()
                .columns([SoundsColumn::Id, SoundsColumn::Src, SoundsColumn::Volume])
                .from(SoundsTable)
                .and_where(Expr::col(SoundsColumn::Id).eq(id)),
        )
        .await
    }

    /// Find sounds with IDs present in the provided list
    pub async fn get_by_ids(db: &DbPool, ids: &[Uuid]) -> DbResult<Vec<SoundModel>> {
        sql_query_all(
            db,
            Query::select()
                .columns(SoundModel::columns())
                .from(SoundsTable)
                .and_where(Expr::col(SoundsColumn::Id).is_in(ids.iter().copied())),
        )
        .await
    }

    /// Find sounds with IDs present in the provided list
    pub async fn get_by_ids_partial(db: &DbPool, ids: &[Uuid]) -> DbResult<Vec<PartialSoundModel>> {
        sql_query_all(
            db,
            Query::select()
                .columns([SoundsColumn::Id, SoundsColumn::Src, SoundsColumn::Volume])
                .from(SoundsTable)
                .and_where(Expr::col(SoundsColumn::Id).is_in(ids.iter().copied())),
        )
        .await
    }

    /// Find all sounds
    pub async fn all(db: &DbPool) -> DbResult<Vec<SoundModel>> {
        sql_query_all(
            db,
            Query::select()
                .columns(SoundModel::columns())
                .from(SoundsTable)
                .order_by_columns([
                    (SoundsColumn::Order, Order::Asc),
                    (SoundsColumn::CreatedAt, Order::Desc),
                ]),
        )
        .await
    }

    /// Find all sounds with a matching name, optionally ignoring case
    pub async fn get_by_names(
        db: &DbPool,
        names: &[String],
        ignore_case: bool,
    ) -> DbResult<Vec<SoundModel>> {
        let mut select = Query::select();

        select.columns(SoundModel::columns()).from(SoundsTable);

        if ignore_case {
            select.and_where(
                // Convert stored name to lower case
                Expr::expr(Func::lower(Expr::col(SoundsColumn::Name)))
                    // Compare with lowercase value
                    .is_in(names.iter().map(|value| value.to_lowercase())),
            );
        } else {
            select.and_where(Expr::col(SoundsColumn::Name).is_in(names));
        };

        sql_query_all(db, &select).await
    }

    /// Update the current sound
    pub async fn update(&mut self, db: &DbPool, data: UpdateSound) -> DbResult<()> {
        let mut update = Query::update();
        update
            .table(SoundsTable)
            .and_where(Expr::col(SoundsColumn::Id).eq(self.id));

        if let Some(name) = data.name {
            self.name = name.clone();
            update.value(SoundsColumn::Name, Expr::value(name));
        }

        if let Some(src) = data.src {
            self.src = src.clone();
            update.value(SoundsColumn::Src, Expr::value(src));
        }

        if let Some(volume) = data.volume {
            self.volume = volume;
            update.value(SoundsColumn::Volume, Expr::value(volume));
        }

        if let Some(order) = data.order {
            self.order = order;
            update.value(SoundsColumn::Order, Expr::value(order));
        }

        sql_exec(db, &update).await
    }

    pub async fn update_order(db: &DbPool, data: Vec<UpdateOrdering>) -> DbResult<()> {
        for order_chunk in data.chunks(1000) {
            let mut case = CaseStatement::new()
                // Use the current column value when not specified
                .finally(Expr::col(SoundsColumn::Order));

            // Add case for all updated values
            for order in order_chunk {
                case = case.case(
                    Expr::col(SoundsColumn::Id).eq(order.id),
                    Expr::value(order.order),
                );
            }

            sql_exec(
                db,
                Query::update()
                    .table(SoundsTable)
                    .value(SoundsColumn::Order, case),
            )
            .await?;
        }

        Ok(())
    }

    pub async fn delete(self, db: &DbPool) -> DbResult<()> {
        sql_exec(
            db,
            Query::delete()
                .from_table(SoundsTable)
                .and_where(Expr::col(SoundsColumn::Id).eq(self.id)),
        )
        .await
    }
}
