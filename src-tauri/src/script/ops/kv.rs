use crate::{
    database::entity::key_value::{CreateKeyValue, KeyValueModel, KeyValueType},
    script::runtime::ScriptRuntimeDataExt,
};
use deno_core::*;
use std::{cell::RefCell, rc::Rc};

#[op2(async)]
#[string]
pub async fn op_kv_get(
    state: Rc<RefCell<OpState>>,
    #[string] key: String,
) -> anyhow::Result<Option<String>> {
    let db = state.db()?;
    let key_value = KeyValueModel::get_by_key(&db, &key).await?;
    let value = key_value.map(|value| value.value);
    Ok(value)
}

#[op2(async)]
#[string]
pub async fn op_kv_remove(
    state: Rc<RefCell<OpState>>,
    #[string] key: String,
) -> anyhow::Result<()> {
    let db = state.db()?;
    KeyValueModel::delete_by_key(&db, &key).await?;
    Ok(())
}

#[op2(async)]
pub async fn op_kv_set(
    state: Rc<RefCell<OpState>>,
    #[string] ty: String,
    #[string] key: String,
    #[string] value: String,
) -> anyhow::Result<()> {
    let db = state.db()?;
    let ty = serde_json::from_str::<KeyValueType>(&format!("\"{ty}\""))?;
    KeyValueModel::create(&db, CreateKeyValue { key, value, ty }).await?;
    Ok(())
}
