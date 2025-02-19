use chrono::{DateTime, Utc};
use sea_query::Value;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub enum MinimumRequireRole {
    #[default]
    None,
    Follower,
    Vip,
    Mod,
    Broadcaster,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MinMax<T> {
    /// Minimum value
    pub min: T,
    /// Maximum value
    pub max: T,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[repr(i32)]
pub enum LoggingLevelDb {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

impl From<LoggingLevelDb> for Value {
    fn from(x: LoggingLevelDb) -> Value {
        let value: i32 = x as i32;
        Value::Int(Some(value))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogsQuery {
    pub level: Option<LoggingLevelDb>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub offset: Option<u64>,
    pub limit: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionsQuery {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub offset: Option<u64>,
    pub limit: Option<u64>,
}

#[derive(Default, Deserialize)]
pub struct UpdateOrdering {
    pub id: Uuid,
    pub order: u32,
}
