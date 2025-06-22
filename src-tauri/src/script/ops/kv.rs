use crate::{
    database::entity::key_value::{CreateKeyValue, KeyValueModel, KeyValueType},
    script::runtime::ScriptRuntimeDataExt,
};
use deno_core::{op2, OpState};
use deno_error::JsErrorBox;
use std::{cell::RefCell, rc::Rc};

#[op2(async)]
#[string]
pub async fn op_kv_get(
    state: Rc<RefCell<OpState>>,
    #[string] key: String,
) -> Result<Option<String>, JsErrorBox> {
    let db = state.db()?;
    let key_value = KeyValueModel::get_by_key(&db, &key).await.map_err(|err| {
        log::error!("failed to load key from database: {err}");
        JsErrorBox::generic("failed to load key from database")
    })?;
    let value = key_value.map(|value| value.value);
    Ok(value)
}

#[op2(async)]
#[string]
pub async fn op_kv_remove(
    state: Rc<RefCell<OpState>>,
    #[string] key: String,
) -> Result<(), JsErrorBox> {
    let db = state.db()?;
    KeyValueModel::delete_by_key(&db, &key)
        .await
        .map_err(|err| {
            log::error!("failed to delete key from database: {err}");
            JsErrorBox::generic("failed to delete key from database")
        })?;
    Ok(())
}

#[op2(async)]
pub async fn op_kv_set(
    state: Rc<RefCell<OpState>>,
    #[string] ty: String,
    #[string] key: String,
    #[string] value: String,
) -> Result<(), JsErrorBox> {
    let db = state.db()?;
    let ty = serde_json::from_str::<KeyValueType>(&format!("\"{ty}\"")).map_err(|err| {
        log::error!("failed to parse key value data: {err}");
        JsErrorBox::generic("failed to parse key value data")
    })?;
    KeyValueModel::create(&db, CreateKeyValue { key, value, ty })
        .await
        .map_err(|err| {
            log::error!("failed to create kv in database: {err}");
            JsErrorBox::generic("failed to kv in database")
        })?;
    Ok(())
}
