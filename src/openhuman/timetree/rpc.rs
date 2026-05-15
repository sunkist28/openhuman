//! JSON-RPC handlers for the `timetree` domain.

use serde_json::{Map, Value};

use crate::openhuman::config::rpc as config_rpc;

use super::ops;

/// Handle `openhuman.timetree_list_calendars`.
pub async fn handle_list_calendars(params: Map<String, Value>) -> Result<Value, String> {
    let access_token = required_str(&params, "access_token")?;
    let outcome = ops::list_calendars(&access_token).await?;
    outcome.into_cli_compatible_json()
}

/// Handle `openhuman.timetree_list_events`.
pub async fn handle_list_events(params: Map<String, Value>) -> Result<Value, String> {
    let access_token = required_str(&params, "access_token")?;
    let calendar_id = required_str(&params, "calendar_id")?;
    let days = optional_u32(&params, "days").unwrap_or(30);
    let timezone = optional_string(&params, "timezone").unwrap_or_else(|| "UTC".to_string());

    let outcome = ops::list_events(&access_token, &calendar_id, days, &timezone).await?;
    outcome.into_cli_compatible_json()
}

/// Handle `openhuman.timetree_sync_to_google_calendar`.
pub async fn handle_sync_to_google_calendar(params: Map<String, Value>) -> Result<Value, String> {
    let config = config_rpc::load_config_with_timeout().await?;
    let access_token = required_str(&params, "access_token")?;
    let calendar_id = required_str(&params, "calendar_id")?;
    let days = optional_u32(&params, "days").unwrap_or(30);
    let timezone = optional_string(&params, "timezone").unwrap_or_else(|| "UTC".to_string());
    let google_calendar_id =
        optional_string(&params, "google_calendar_id").unwrap_or_else(|| "primary".to_string());

    let outcome = ops::sync_to_google_calendar(
        &config,
        &access_token,
        &calendar_id,
        days,
        &timezone,
        &google_calendar_id,
    )
    .await?;
    outcome.into_cli_compatible_json()
}

// ── param helpers ────────────────────────────────────────────────────────────

fn required_str(params: &Map<String, Value>, key: &str) -> Result<String, String> {
    match params.get(key) {
        Some(Value::String(s)) if !s.trim().is_empty() => Ok(s.clone()),
        Some(_) => Err(format!("[timetree] '{key}' must be a non-empty string")),
        None => Err(format!("[timetree] missing required parameter '{key}'")),
    }
}

fn optional_string(params: &Map<String, Value>, key: &str) -> Option<String> {
    params
        .get(key)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn optional_u32(params: &Map<String, Value>, key: &str) -> Option<u32> {
    params
        .get(key)
        .and_then(|v| v.as_u64())
        .map(|n| n.min(u32::MAX as u64) as u32)
}
