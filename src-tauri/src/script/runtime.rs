use crate::{
    database::DbPool,
    events::matching::{EventData, EventInputData},
    overlay::OverlayMessageSender,
    script::ops::{
        core::{op_sleep, op_uuid_v4},
        http::op_http_request,
        kv::{op_kv_get, op_kv_remove, op_kv_set},
        logging::op_log,
        twitch::op_twitch_get_credentials,
        vtftk::{
            op_vtftk_emit_overlay_message, op_vtftk_get_items_by_ids, op_vtftk_get_items_by_names,
            op_vtftk_get_sounds_by_ids, op_vtftk_get_sounds_by_names,
        },
    },
    twitch::manager::Twitch,
};
use anyhow::Context;
use deno_core::{
    serde_v8::to_v8,
    v8::{self, Global, Local},
    JsRuntime, OpState, PollEventLoopOptions, RuntimeOptions,
};
use deno_error::JsErrorBox;
use serde::{Deserialize, Serialize};
use std::{
    cell::{Ref, RefCell},
    future::Future,
    path::PathBuf,
    pin::Pin,
    rc::Rc,
    task::Poll,
};
use tokio::{
    sync::{mpsc, oneshot},
    task::LocalSet,
};
use twitch_api::types::{DisplayName, UserId, UserName};
use uuid::Uuid;

use super::module_loader::AppModuleLoader;

pub struct ScriptRuntimeData {
    /// Sender handle for sending messages to the overlay
    pub overlay_sender: OverlayMessageSender,

    /// Access to the database
    pub db: DbPool,

    /// Access to the twitch manager
    pub twitch: Twitch,
}

deno_core::extension!(
    api_extension,
    ops = [
        // Core
        op_uuid_v4,
        op_sleep,
        // HTTP
        op_http_request,
        // Logging
        op_log,
        // Twitch
        op_twitch_get_credentials,
        // KV
        op_kv_get,
        op_kv_set,
        op_kv_remove,
        // VTFTK Sounds
        op_vtftk_get_sounds_by_names,
        op_vtftk_get_sounds_by_ids,
        // VTFTK Items
        op_vtftk_get_items_by_names,
        op_vtftk_get_items_by_ids,
        // VTFTK Overlay
        op_vtftk_emit_overlay_message,
    ],
    options = {
        data: ScriptRuntimeData
    },
    state = |state, options| { state.put(options.data); },
    docs = "Extension providing APIs to the JS runtime"
);

/// Snapshot of the script engine runtime, see [build.rs](../../build.rs)
static SCRIPT_RUNTIME_SNAPSHOT: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/SCRIPT_RUNTIME_SNAPSHOT.bin"));

/// Context passed to the JS runtime that is tracked
/// across async calls for handling logging sources
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RuntimeExecutionContext {
    /// Runtime execution started from a event
    Event { event_id: Uuid },
    /// Runtime execution started from a command
    Command { command_id: Uuid },
}

#[derive(Debug)]
pub enum ScriptExecutorMessage {
    /// Tell the executor to run the event callbacks in the provided code
    /// on the runtime
    EventScript {
        /// Context for logging
        ctx: RuntimeExecutionContext,
        /// The script code to run
        script: String,
        /// Data for the event
        data: EventData,
        /// Channel to send back the result
        tx: oneshot::Sender<anyhow::Result<()>>,
    },

    /// Tell the executor to run the event callbacks in the provided code
    /// on the runtime
    CommandScript {
        /// Context for logging
        ctx: RuntimeExecutionContext,
        /// The script code to run
        script: String,
        /// Context for the command run
        cmd_ctx: CommandContext,
        /// Channel to send back the result
        tx: oneshot::Sender<anyhow::Result<()>>,
    },
}

/// Handle for accessing the script executor
#[derive(Clone)]
pub struct ScriptExecutorHandle {
    /// Channel for sending the execute message
    tx: mpsc::Sender<ScriptExecutorMessage>,
}

impl ScriptExecutorHandle {
    /// Execute the provided `script` using `event` on the runtime this handle
    /// is linked to, returning the result
    pub async fn execute(
        &self,
        ctx: RuntimeExecutionContext,
        script: String,
        data: EventData,
    ) -> anyhow::Result<()> {
        let (tx, rx) = oneshot::channel();

        self.tx
            .send(ScriptExecutorMessage::EventScript {
                ctx,
                script,
                data,
                tx,
            })
            .await
            .context("executor is not running")?;

        rx.await.context("executor closed without response")?
    }

    pub async fn execute_command(
        &self,
        ctx: RuntimeExecutionContext,
        script: String,
        cmd_ctx: CommandContext,
    ) -> anyhow::Result<()> {
        let (tx, rx) = oneshot::channel();

        self.tx
            .send(ScriptExecutorMessage::CommandScript {
                ctx,
                script,
                cmd_ctx,
                tx,
            })
            .await
            .context("executor is not running")?;

        rx.await.context("executor closed without response")?
    }
}

fn spawn_script_promise(
    js_runtime: &mut JsRuntime,
    global_promise: anyhow::Result<v8::Global<v8::Value>>,
    tx: oneshot::Sender<anyhow::Result<()>>,
    local_set: &mut LocalSet,
) {
    let global_promise = match global_promise {
        Ok(value) => value,
        Err(err) => {
            _ = tx.send(Err(err));
            return;
        }
    };

    let resolve = js_runtime.resolve(global_promise);
    local_set.spawn_local(async move {
        let result = resolve.await;
        _ = tx.send(result.map(|_| ()).map_err(anyhow::Error::new));
    });
}

/// Creates a dedicated thread for receiving script execution requests. The
/// thread will process the script execution requests providing the responses
///
/// The JS runtime is !Send and thus it cannot be shared across tokio async tasks
/// so here its provided a dedicated single threaded runtime and its own thread
pub fn create_script_executor(
    modules_path: PathBuf,
    runtime_data: ScriptRuntimeData,
) -> ScriptExecutorHandle {
    let (tx, rx) = mpsc::channel::<ScriptExecutorMessage>(5);

    std::thread::spawn(move || {
        // Create a new tokio runtime in the dedicated thread
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to create script async runtime");

        // Create runtime
        let js_runtime = JsRuntime::new(RuntimeOptions {
            startup_snapshot: Some(SCRIPT_RUNTIME_SNAPSHOT),
            extensions: vec![api_extension::init(runtime_data)],
            module_loader: Some(Rc::new(AppModuleLoader {
                module_root: modules_path,
            })),

            ..Default::default()
        });

        runtime.block_on(ScriptExecutorFuture::new(js_runtime, rx));
    });

    ScriptExecutorHandle { tx }
}

struct ScriptExecutorFuture {
    /// JS runtime task
    runtime: JsRuntime,

    /// Channel to receive execute messages from
    rx: mpsc::Receiver<ScriptExecutorMessage>,

    /// Local set for spawned promise tasks
    local_set: LocalSet,
}

impl ScriptExecutorFuture {
    pub fn new(runtime: JsRuntime, rx: mpsc::Receiver<ScriptExecutorMessage>) -> Self {
        Self {
            runtime,
            rx,
            local_set: LocalSet::new(),
        }
    }
}

impl Future for ScriptExecutorFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        // Initial pass when not messages are available
        {
            // Poll the promises local set
            _ = Pin::new(&mut this.local_set).poll(cx);

            // Poll event loop for any promises
            let _ = this
                .runtime
                .poll_event_loop(cx, PollEventLoopOptions::default());
        }

        // Poll incoming script execute messages
        while let Poll::Ready(msg) = this.rx.poll_recv(cx) {
            let msg = match msg {
                Some(msg) => msg,
                None => return Poll::Ready(()),
            };

            match msg {
                ScriptExecutorMessage::EventScript {
                    ctx,
                    script,
                    data,
                    tx,
                } => {
                    let result = execute_script(&mut this.runtime, ctx, script, data);
                    spawn_script_promise(&mut this.runtime, result, tx, &mut this.local_set)
                }
                ScriptExecutorMessage::CommandScript {
                    ctx,
                    script,
                    cmd_ctx,
                    tx,
                } => {
                    let result = execute_command(&mut this.runtime, ctx, script, cmd_ctx);
                    spawn_script_promise(&mut this.runtime, result, tx, &mut this.local_set)
                }
            }

            // Poll the promises local set
            _ = Pin::new(&mut this.local_set).poll(cx);

            // Poll the event loop
            let _ = this
                .runtime
                .poll_event_loop(cx, PollEventLoopOptions::default());
        }

        Poll::Pending
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandContext {
    pub message_id: String,
    pub full_message: String,
    pub message: String,
    pub user: CommandContextUser,
    pub args: Vec<String>,
    pub input_data: EventInputData,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandContextUser {
    pub id: UserId,
    pub name: UserName,
    pub display_name: DisplayName,
}

/// Executes the provided command
///
/// Returns a promise value that resolves when the command is complete
fn execute_command(
    runtime: &mut JsRuntime,
    ctx: RuntimeExecutionContext,
    script: String,
    cmd_ctx: CommandContext,
) -> anyhow::Result<v8::Global<v8::Value>> {
    // Get the handle scope
    let scope = &mut runtime.handle_scope();

    // Wrap code in async function to allow await
    let code = format!("async (ctx) => {{ {script} }}");

    // Eval user code to create the async function
    let event_fn: Local<'_, v8::Function> =
        JsRuntime::eval(scope, &code).context("failed to create script function")?;

    // Get the global object
    let global = scope.get_current_context().global(scope);

    // Create object keys
    let api_key = to_v8(scope, "api")?;
    let internal_key = to_v8(scope, "internal")?;
    let execute_command_outlet_key = to_v8(scope, "executeCommandOutlet")?;

    // Get API object
    let api: Local<'_, v8::Object> = global
        .get(scope, api_key)
        .context("api unavailable")?
        .try_cast()?;

    // Get internal API object
    let internal: Local<'_, v8::Object> = api
        .get(scope, internal_key)
        .context("internal api unavailable")?
        .try_cast()?;

    // Get executeCommandOutlet function
    let execute_command_outlet: Local<'_, v8::Function> = internal
        .get(scope, execute_command_outlet_key)
        .context("executeCommandOutlet missing")?
        .try_cast()?;

    let global_value = global.try_cast()?;
    let ctx_value = to_v8(scope, ctx)?;
    let cmd_ctx_value = to_v8(scope, cmd_ctx)?;
    let event_fn_value = event_fn.try_cast()?;

    let result = execute_command_outlet
        .call(
            scope,
            global_value,
            &[ctx_value, cmd_ctx_value, event_fn_value],
        )
        .context("function provided no return value")?;

    Ok(Global::new(scope, result))
}

/// Executes the provided script using the provided event
///
/// Returns a promise value that resolves when the script is complete
fn execute_script(
    runtime: &mut JsRuntime,
    ctx: RuntimeExecutionContext,
    script: String,
    data: EventData,
) -> anyhow::Result<v8::Global<v8::Value>> {
    // Get the handle scope
    let scope = &mut runtime.handle_scope();

    // Wrap code in async function to allow await
    let code = format!("async (event) => {{ {script} }}");

    // Eval user code to create the async function
    let event_fn: Local<'_, v8::Function> =
        JsRuntime::eval(scope, &code).context("failed to create script function")?;

    // Get the global object
    let global = scope.get_current_context().global(scope);

    // Create object keys
    let api_key = to_v8(scope, "api")?;
    let internal_key = to_v8(scope, "internal")?;
    let execute_event_outlet_key = to_v8(scope, "executeEventOutlet")?;

    // Get API object
    let api: Local<'_, v8::Object> = global
        .get(scope, api_key)
        .context("api unavailable")?
        .try_cast()?;

    // Get internal API object
    let internal: Local<'_, v8::Object> = api
        .get(scope, internal_key)
        .context("internal api unavailable")?
        .try_cast()?;

    // Get executeEventOutlet function
    let execute_event_outlet: Local<'_, v8::Function> = internal
        .get(scope, execute_event_outlet_key)
        .context("executeEventOutlet missing")?
        .try_cast()?;

    let global_value = global.try_cast()?;
    let ctx_value = to_v8(scope, ctx)?;
    let data_value = to_v8(scope, data)?;
    let event_fn_value = event_fn.try_cast()?;

    let result = execute_event_outlet
        .call(
            scope,
            global_value,
            &[ctx_value, data_value, event_fn_value],
        )
        .context("function provided no return value")?;

    Ok(Global::new(scope, result))
}

/// Helper extension to extract script runtime fields
/// from the shared OpState ref
pub trait ScriptRuntimeDataExt {
    fn try_borrow_state(&self) -> Result<Ref<'_, OpState>, JsErrorBox>;

    fn overlay_sender(&self) -> Result<OverlayMessageSender, JsErrorBox>;
    fn db(&self) -> Result<DbPool, JsErrorBox>;
    fn twitch(&self) -> Result<Twitch, JsErrorBox>;
}

impl ScriptRuntimeDataExt for Rc<RefCell<OpState>> {
    fn try_borrow_state(&self) -> Result<Ref<'_, OpState>, JsErrorBox> {
        self.try_borrow().map_err(|err| {
            log::error!("failed to get op state: {err}");
            JsErrorBox::generic("failed to get op state")
        })
    }

    fn overlay_sender(&self) -> Result<OverlayMessageSender, JsErrorBox> {
        let state = self.try_borrow_state()?;
        let data = state.borrow::<ScriptRuntimeData>();
        Ok(data.overlay_sender.clone())
    }

    fn db(&self) -> Result<DbPool, JsErrorBox> {
        let state = self.try_borrow_state()?;
        let data = state.borrow::<ScriptRuntimeData>();
        Ok(data.db.clone())
    }

    fn twitch(&self) -> Result<Twitch, JsErrorBox> {
        let state = self.try_borrow_state()?;
        let data = state.borrow::<ScriptRuntimeData>();
        Ok(data.twitch.clone())
    }
}
