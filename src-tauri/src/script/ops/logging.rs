//! # Logging (JS API)
//!
//! API for performing logging from the JS runtime

use std::{cell::RefCell, rc::Rc};

use chrono::Utc;
use deno_core::*;
use log::error;

use crate::{
    database::entity::{
        command_log::{CommandLogsModel, CreateCommandLog},
        event_log::{CreateEventLog, EventLogsModel},
        shared::LoggingLevelDb,
    },
    script::runtime::{RuntimeExecutionContext, ScriptRuntimeDataExt},
};

#[op2]
pub fn op_log(
    state: Rc<RefCell<OpState>>,
    #[serde] ctx: Option<RuntimeExecutionContext>,
    #[serde] level: LoggingLevelDb,
    #[string] message: String,
) -> anyhow::Result<()> {
    let db = state.db()?;

    let prefix = match ctx {
        Some(ctx) => match ctx {
            RuntimeExecutionContext::Event { event_id } => format!("[event:{event_id}]"),
            RuntimeExecutionContext::Command { command_id } => format!("[command:{command_id}]"),
        },
        None => "[unknown]".to_string(),
    };

    let log_level = match &level {
        LoggingLevelDb::Debug => log::Level::Debug,
        LoggingLevelDb::Info => log::Level::Info,
        LoggingLevelDb::Warn => log::Level::Warn,
        LoggingLevelDb::Error => log::Level::Error,
    };

    log::log!(log_level, "{prefix}: {message}");

    if let Some(ctx) = ctx {
        let created_at = Utc::now();

        tokio::spawn(async move {
            let result = match ctx {
                RuntimeExecutionContext::Event { event_id } => {
                    EventLogsModel::create(
                        &db,
                        CreateEventLog {
                            event_id,
                            level,
                            message,
                            created_at,
                        },
                    )
                    .await
                }
                RuntimeExecutionContext::Command { command_id } => {
                    CommandLogsModel::create(
                        &db,
                        CreateCommandLog {
                            command_id,
                            level,
                            message,
                            created_at,
                        },
                    )
                    .await
                }
            };

            if let Err(err) = result {
                error!("failed to persist log: {:?}", err);
            }
        });
    }

    Ok(())
}
