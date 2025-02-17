//! # Logging (JS API)
//!
//! API for performing logging from the JS runtime

use std::{cell::RefCell, rc::Rc};

use chrono::Utc;
use deno_core::*;
use log::error;

use crate::{
    database::entity::{
        commands::{CommandModel, CreateCommandLog},
        events::{CreateEventLog, EventModel},
        shared::LoggingLevelDb,
    },
    script::runtime::{RuntimeExecutionContext, ScriptRuntimeData},
};

fn exec_prefix(ctx: Option<&RuntimeExecutionContext>) -> String {
    match ctx {
        Some(ctx) => match ctx {
            RuntimeExecutionContext::Event { event_id } => format!("[event:{event_id}]"),
            RuntimeExecutionContext::Command { command_id } => format!("[command:{command_id}]"),
        },
        None => "[unknown]".to_string(),
    }
}

#[op2]
pub fn op_log(
    state: Rc<RefCell<OpState>>,
    #[serde] ctx: Option<RuntimeExecutionContext>,
    #[serde] level: LoggingLevelDb,
    #[string] message: String,
) {
    let db = {
        let state = state.borrow();
        let data = state.borrow::<ScriptRuntimeData>();
        data.db.clone()
    };

    let prefix = exec_prefix(ctx.as_ref());

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
            match ctx {
                RuntimeExecutionContext::Event { event_id } => {
                    if let Err(err) = EventModel::create_log(
                        &db,
                        CreateEventLog {
                            event_id,
                            level,
                            message,
                            created_at,
                        },
                    )
                    .await
                    {
                        error!("failed to persist script log: {:?}", err);
                    }
                }
                RuntimeExecutionContext::Command { command_id } => {
                    if let Err(err) = CommandModel::create_log(
                        &db,
                        CreateCommandLog {
                            command_id,
                            level,
                            message,
                            created_at,
                        },
                    )
                    .await
                    {
                        error!("failed to persist command log: {:?}", err);
                    }
                }
            };
        });
    }
}
