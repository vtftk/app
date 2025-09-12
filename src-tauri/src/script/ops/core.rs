use std::time::Duration;

use deno_core::op2;
use deno_error::JsErrorBox;
use tokio::time::sleep;
use uuid::Uuid;

/// Generates a random UUID returning it in string form for the JS
/// scripting engine
#[op2]
#[string]
pub fn op_uuid_v4() -> String {
    Uuid::new_v4().to_string()
}

/// Sleep for some duration in milliseconds
#[op2(async)]
pub async fn op_sleep(duration_ms: u32) -> Result<(), JsErrorBox> {
    sleep(Duration::from_millis(duration_ms as u64)).await;
    Ok(())
}
