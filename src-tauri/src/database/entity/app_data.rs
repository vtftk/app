use anyhow::Context;
use chrono::Utc;
use sea_orm::{
    entity::prelude::*, sea_query::OnConflict, ActiveValue::Set, FromJsonQueryResult,
    FromQueryResult, QuerySelect,
};
use serde::{Deserialize, Serialize};
use twitch_api::{helix::Scope, twitch_oauth2::AccessToken};

// Type alias helpers for the database entity types
pub type AppDataModel = Model;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "app_data")]
pub struct Model {
    /// Unique ID for the sound
    #[sea_orm(primary_key)]
    pub id: i32,

    pub data: AppData,

    pub created_at: DateTimeUtc,
    pub last_modified_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, Serialize, Deserialize, Default, FromJsonQueryResult)]
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

#[derive(Debug, Clone, Serialize, Deserialize, FromJsonQueryResult)]
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

impl MainConfig {
    pub fn get_http_port(&self) -> u16 {
        #[cfg(not(debug_assertions))]
        return self.http_port;

        // Debug fixed port override
        #[cfg(debug_assertions)]
        return 58372;
    }
}

pub fn default_http_port() -> u16 {
    58371
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
    pub duration: f32,
    /// Range of speed a thrown object can have
    pub spin_speed: MinMax<f32>,
    /// Range of angles an object can be thrown at
    pub throw_angle: MinMax<f32>,
    /// Which direction objects should come from
    pub direction: ThrowDirection,
    /// Delay in milliseconds before impacts show up
    pub impact_delay: f32,
    /// Item scale, range relative to the scale of the model
    pub item_scale: MinMax<f32>,
}

impl Default for ThrowablesConfig {
    fn default() -> Self {
        Self {
            duration: 1000.,
            spin_speed: MinMax {
                min: 300.,
                max: 750.,
            },
            throw_angle: MinMax {
                min: -45.,
                max: 45.,
            },
            direction: ThrowDirection::default(),
            impact_delay: 100.,
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
    pub model_return_time: f32,

    /// How eyes should react when the model is hit by a throwable
    pub eyes_on_hit: EyesMode,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model_return_time: 300.,
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

impl Model {
    /// Only one row should ever be created and should have this ID
    const SINGLETON_ID: i32 = 1;

    /// Create a new sound
    pub async fn set<C>(db: &C, app_data: AppData) -> anyhow::Result<Model>
    where
        C: ConnectionTrait + Send + 'static,
    {
        let active_model = ActiveModel {
            id: Set(Self::SINGLETON_ID),
            data: Set(app_data),
            created_at: Set(Utc::now()),
            last_modified_at: Set(Utc::now()),
        };

        Entity::insert(active_model)
            .on_conflict(
                OnConflict::column(Column::Id)
                    .update_columns([Column::Data, Column::LastModifiedAt])
                    .to_owned(),
            )
            .exec_without_returning(db)
            .await?;

        let model = Entity::find_by_id(Self::SINGLETON_ID)
            .one(db)
            .await?
            .context("model not inserted")?;

        Ok(model)
    }

    pub async fn get_or_default<C>(db: &C) -> anyhow::Result<AppData>
    where
        C: ConnectionTrait + Send + 'static,
    {
        let model = Entity::find_by_id(Self::SINGLETON_ID).one(db).await?;

        let model = match model {
            Some(value) => value,
            None => Self::set(db, Default::default()).await?,
        };

        Ok(model.data)
    }

    /// HTTP port is loaded pretty frequently
    pub async fn get_http_port<C>(db: &C) -> anyhow::Result<u16>
    where
        C: ConnectionTrait + Send + 'static,
    {
        #[derive(Default, FromQueryResult)]
        struct PartialModel {
            http_port: Option<u16>,
        }

        // HTTP port is loaded frequently so save on loading the entire main_config every time
        let http_port = Entity::find_by_id(Self::SINGLETON_ID)
            .select_only()
            // Select just the HTTP port from the data
            .expr_as(
                Expr::cust("json_extract(data, '$.main_config.http_port')"),
                "main_config",
            )
            .into_model::<PartialModel>()
            .one(db)
            .await?
            .and_then(|value| value.http_port)
            .unwrap_or_else(default_http_port);

        // Debug fixed port override
        #[cfg(debug_assertions)]
        {
            _ = http_port;
            Ok(58372)
        }

        #[cfg(not(debug_assertions))]
        Ok(http_port)
    }

    pub async fn get_main_config<C>(db: &C) -> anyhow::Result<MainConfig>
    where
        C: ConnectionTrait + Send + 'static,
    {
        #[derive(Default, FromQueryResult)]
        struct PartialModel {
            main_config: MainConfig,
        }

        Ok(Entity::find_by_id(Self::SINGLETON_ID)
            .select_only()
            // Select
            .expr_as(
                Expr::cust("json_extract(data, '$.main_config')"),
                "main_config",
            )
            .into_model::<PartialModel>()
            .one(db)
            .await?
            .map(|value| value.main_config)
            .unwrap_or_default())
    }
}
