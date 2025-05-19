use chrono::{DateTime, Utc};
use sea_query::{Alias, Expr, Func, IdenStatic, OnConflict, Query};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use twitch_api::{helix::Scope, twitch_oauth2::AccessToken};

use crate::database::{
    helpers::{sql_exec, sql_query_maybe_one},
    DbPool,
};

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct AppDataModel {
    pub id: i32,
    #[sqlx(json)]
    pub data: AppData,
    pub created_at: DateTime<Utc>,
    pub last_modified_at: DateTime<Utc>,
}

#[derive(IdenStatic, Copy, Clone)]
#[iden(rename = "app_data")]
pub struct AppDataTable;

#[derive(IdenStatic, Copy, Clone)]
pub enum AppDataColumn {
    Id,
    Data,
    CreatedAt,
    LastModifiedAt,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AppData {
    #[serde(flatten)]
    pub app: AppConfig,

    #[serde(flatten)]
    pub overlay: OverlayConfig,
}

impl PartialEq for AppData {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AppConfig {
    pub main_config: MainConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct OverlayConfig {
    pub throwables_config: ThrowablesConfig,
    pub model_config: ModelConfig,
    pub sounds_config: SoundsConfig,
    pub vtube_studio_config: VTubeStudioConfig,
    pub physics_config: PhysicsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MainConfig {
    /// Minimize to try instead of closing
    pub minimize_to_tray: bool,
    /// Clean old log data on startup
    pub clean_logs: bool,
    /// Number of days of logs to retain when cleaning logs
    pub clean_logs_days: u64,
    /// Clean old execution data on start
    pub clean_executions: bool,
    /// Number of days of execution data to retain when cleaning executions
    pub clean_executions_days: u64,
    /// Clean old chat history data on start
    pub clean_chat_history: bool,
    /// Number of days of chat history data to retain when cleaning executions
    pub clean_chat_history_days: u64,
    /// Allow automatic updates
    pub auto_updating: bool,
    /// Port for the HTTP server
    http_port: u16,
}

pub fn default_http_port() -> u16 {
    8533
}

impl Default for MainConfig {
    fn default() -> Self {
        Self {
            minimize_to_tray: true,
            clean_logs: true,
            clean_logs_days: 30,
            clean_executions: true,
            clean_executions_days: 30,
            clean_chat_history: true,
            clean_chat_history_days: 1,
            auto_updating: true,
            http_port: default_http_port(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VTubeStudioConfig {
    pub host: String,
    pub port: u16,
}

impl Default for VTubeStudioConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8001,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TwitchConfig {
    pub access_token: Option<AccessToken>,
    pub scopes: Option<Vec<Scope>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThrowablesConfig {
    /// Duration in milliseconds that a thrown object should spend
    /// being thrown
    pub duration: u32,
    /// Range of speed a thrown object can have
    pub spin_speed: MinMax<u32>,
    /// Range of angles an object can be thrown at
    pub throw_angle: MinMax<f32>,
    /// Which direction objects should come from
    pub direction: ThrowDirection,
    /// Delay in milliseconds before impacts show up
    pub impact_delay: u32,
    /// Item scale, range relative to the scale of the model
    pub item_scale: MinMax<f32>,
}

impl Default for ThrowablesConfig {
    fn default() -> Self {
        Self {
            duration: 1000,
            spin_speed: MinMax { min: 300, max: 750 },
            throw_angle: MinMax {
                min: -45.,
                max: 45.,
            },
            direction: ThrowDirection::default(),
            impact_delay: 100,
            item_scale: MinMax { min: 0.25, max: 3. },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SoundsConfig {
    /// Global volume for all sounds
    pub global_volume: f32,
}

impl Default for SoundsConfig {
    fn default() -> Self {
        Self { global_volume: 0.5 }
    }
}

/// Determines how the direction for thrown objects is chosen
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ThrowDirection {
    /// Random direction, left or right
    Random,
    /// Random but weighted
    #[default]
    Weighted,
    /// Only thrown from left side
    LeftOnly,
    /// Only thrown from right side
    RightOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinMax<T> {
    /// Minimum value
    pub min: T,
    /// Maximum value
    pub max: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ModelConfig {
    /// Time in seconds the model will take to return to its
    /// original position in milliseconds
    pub model_return_time: u32,

    /// How eyes should react when the model is hit by a throwable
    pub eyes_on_hit: EyesMode,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model_return_time: 300,
            eyes_on_hit: EyesMode::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum EyesMode {
    /// Eyes should not be changed
    #[default]
    Unchanged,
    /// Eyes should be opened
    Opened,
    /// Eyes should be closed
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PhysicsConfig {
    pub enabled: bool,
    pub fps: u16,
    pub gravity_multiplier: f32,
    pub horizontal_multiplier: f32,
    pub vertical_multiplier: f32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            fps: 30,
            gravity_multiplier: 1.,
            horizontal_multiplier: 1.,
            vertical_multiplier: 1.,
        }
    }
}

impl AppDataModel {
    /// Only one row should ever be created and should have this ID
    const SINGLETON_ID: i32 = 1;

    pub async fn set(db: &DbPool, app_data: AppData) -> anyhow::Result<AppDataModel> {
        let model = AppDataModel {
            id: Self::SINGLETON_ID,
            data: app_data,
            created_at: Utc::now(),
            last_modified_at: Utc::now(),
        };

        let data_value = serde_json::to_value(&model.data)?;

        sql_exec(
            db,
            Query::insert()
                .into_table(AppDataTable)
                .columns([
                    AppDataColumn::Id,
                    AppDataColumn::Data,
                    AppDataColumn::CreatedAt,
                    AppDataColumn::LastModifiedAt,
                ])
                .values_panic([
                    model.id.into(),
                    data_value.into(),
                    model.created_at.into(),
                    model.last_modified_at.into(),
                ])
                .on_conflict(
                    OnConflict::new()
                        .update_column(AppDataColumn::Data)
                        .update_column(AppDataColumn::LastModifiedAt)
                        .to_owned(),
                ),
        )
        .await?;

        Ok(model)
    }

    pub async fn get_or_default(db: &DbPool) -> anyhow::Result<AppData> {
        let result: Option<AppDataModel> = sql_query_maybe_one(
            db,
            Query::select()
                .from(AppDataTable)
                .columns([
                    AppDataColumn::Id,
                    AppDataColumn::Data,
                    AppDataColumn::CreatedAt,
                    AppDataColumn::LastModifiedAt,
                ])
                .and_where(Expr::col(AppDataColumn::Id).add(Self::SINGLETON_ID)),
        )
        .await?;

        let model = match result {
            Some(value) => value,
            None => Self::set(db, Default::default()).await?,
        };

        Ok(model.data)
    }

    /// HTTP port is loaded pretty frequently
    #[cfg_attr(debug_assertions, allow(unused))]
    pub async fn get_http_port(db: &DbPool) -> anyhow::Result<u16> {
        let result: Option<(u16,)> = sql_query_maybe_one(
            db,
            Query::select()
                .from(AppDataTable)
                // Select
                .expr(Func::coalesce([
                    Expr::cust("json_extract(data, '$.main_config.http_port')"),
                    Expr::value(default_http_port()),
                ]))
                .and_where(Expr::col(AppDataColumn::Id).add(Self::SINGLETON_ID)),
        )
        .await?;

        // HTTP port is loaded frequently so save on loading the entire main_config every time
        let http_port = result.map(|(port,)| port).unwrap_or_else(default_http_port);
        Ok(http_port)
    }

    pub async fn get_main_config(db: &DbPool) -> anyhow::Result<MainConfig> {
        #[derive(FromRow)]
        struct PartialModel {
            #[sqlx(json)]
            main_config: MainConfig,
        }

        let result: Option<PartialModel> = sql_query_maybe_one(
            db,
            Query::select()
                .from(AppDataTable)
                // Select
                .expr_as(
                    Expr::cust("json_extract(data, '$.main_config')"),
                    Alias::new("main_config"),
                )
                .and_where(Expr::col(AppDataColumn::Id).add(Self::SINGLETON_ID)),
        )
        .await?;

        Ok(result.map(|value| value.main_config).unwrap_or_default())
    }
}
