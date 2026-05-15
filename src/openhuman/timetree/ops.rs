//! Domain operations for the `timetree` domain.
//!
//! Three public entry points:
//!  - `list_calendars`              — fetch a user's TimeTree calendars
//!  - `list_events`                 — fetch upcoming events from one calendar
//!  - `sync_to_google_calendar`     — copy TimeTree events into Google Calendar
//!                                    via the Composio `GOOGLECALENDAR_CREATE_EVENT` tool

use serde_json::json;

use crate::openhuman::config::Config;
use crate::rpc::RpcOutcome;

use super::client::TimeTreeClient;
use super::types::{CalendarSummary, EventSummary, SyncResult};

type OpResult<T> = std::result::Result<T, String>;

fn build_timetree_client(access_token: &str) -> OpResult<TimeTreeClient> {
    TimeTreeClient::new(access_token)
        .map_err(|e| format!("[timetree] failed to build client: {e}"))
}

/// Fetch all calendars for the given TimeTree personal access token.
pub async fn list_calendars(
    access_token: &str,
) -> OpResult<RpcOutcome<Vec<CalendarSummary>>> {
    tracing::debug!("[timetree] list_calendars");
    let client = build_timetree_client(access_token)?;
    let resp = client
        .list_calendars()
        .await
        .map_err(|e| e.to_string())?;

    let calendars: Vec<CalendarSummary> = resp.data.into_iter().map(Into::into).collect();
    tracing::info!(count = calendars.len(), "[timetree] list_calendars: fetched {} calendar(s)", calendars.len());
    Ok(RpcOutcome::new(
        calendars,
        vec![],
    ))
}

/// Fetch upcoming events from a specific TimeTree calendar.
pub async fn list_events(
    access_token: &str,
    calendar_id: &str,
    days: u32,
    timezone: &str,
) -> OpResult<RpcOutcome<Vec<EventSummary>>> {
    tracing::debug!(
        calendar_id = %calendar_id,
        days = days,
        "[timetree] list_events"
    );
    let client = build_timetree_client(access_token)?;
    let resp = client
        .list_upcoming_events(calendar_id, days, timezone)
        .await
        .map_err(|e| e.to_string())?;

    let events: Vec<EventSummary> = resp.data.into_iter().map(Into::into).collect();
    tracing::info!(
        calendar_id = %calendar_id,
        count = events.len(),
        "[timetree] list_events: fetched {} event(s)",
        events.len()
    );
    Ok(RpcOutcome::new(events, vec![]))
}

/// Sync upcoming TimeTree events into a Google Calendar via Composio.
///
/// Requires the user to have a connected Google Calendar account in Composio.
/// Each TimeTree event is sent as a `GOOGLECALENDAR_CREATE_EVENT` call.
pub async fn sync_to_google_calendar(
    config: &Config,
    access_token: &str,
    calendar_id: &str,
    days: u32,
    timezone: &str,
    google_calendar_id: &str,
) -> OpResult<RpcOutcome<SyncResult>> {
    tracing::info!(
        calendar_id = %calendar_id,
        days = days,
        google_calendar_id = %google_calendar_id,
        "[timetree] sync_to_google_calendar: starting"
    );

    // 1. Resolve Composio client (requires user to be signed in).
    let composio_client =
        crate::openhuman::composio::client::build_composio_client(config).ok_or_else(|| {
            "[timetree] Google Calendar sync requires a backend session. Sign in first.".to_string()
        })?;

    // 2. Fetch TimeTree events.
    let timetree_client = build_timetree_client(access_token)?;
    let events_resp = timetree_client
        .list_upcoming_events(calendar_id, days, timezone)
        .await
        .map_err(|e| format!("[timetree] failed to fetch events: {e}"))?;

    let events: Vec<EventSummary> = events_resp.data.into_iter().map(Into::into).collect();
    tracing::info!(
        count = events.len(),
        "[timetree] sync_to_google_calendar: fetched {} event(s) from TimeTree",
        events.len()
    );

    // 3. Create each event in Google Calendar.
    let mut synced = 0usize;
    let mut skipped = 0usize;
    let mut errors: Vec<String> = Vec::new();

    for event in &events {
        let arguments = build_gcal_event_arguments(event, google_calendar_id);
        tracing::debug!(
            event_id = %event.id,
            title = %event.title,
            "[timetree] creating GCal event for TimeTree event"
        );

        match composio_client
            .execute_tool("GOOGLECALENDAR_CREATE_EVENT", Some(arguments))
            .await
        {
            Ok(resp) if resp.successful => {
                tracing::debug!(
                    event_id = %event.id,
                    "[timetree] GCal event created successfully"
                );
                synced += 1;
            }
            Ok(resp) => {
                let msg = resp
                    .error
                    .unwrap_or_else(|| "unknown error from Google Calendar".to_string());
                tracing::warn!(
                    event_id = %event.id,
                    error = %msg,
                    "[timetree] GCal create_event reported failure"
                );
                errors.push(format!("event '{}' ({}): {}", event.title, event.id, msg));
                skipped += 1;
            }
            Err(e) => {
                tracing::error!(
                    event_id = %event.id,
                    error = %e,
                    "[timetree] GCal create_event request failed"
                );
                errors.push(format!("event '{}' ({}): {}", event.title, event.id, e));
                skipped += 1;
            }
        }
    }

    tracing::info!(
        synced = synced,
        skipped = skipped,
        errors = errors.len(),
        "[timetree] sync_to_google_calendar: finished"
    );

    let result = SyncResult {
        synced_count: synced,
        skipped_count: skipped,
        errors,
    };
    Ok(RpcOutcome::new(
        result,
        vec![format!("[timetree] synced {synced} event(s), skipped {skipped}")],
    ))
}

/// Build the Composio arguments map for `GOOGLECALENDAR_CREATE_EVENT`.
fn build_gcal_event_arguments(event: &EventSummary, calendar_id: &str) -> serde_json::Value {
    let tz = event
        .start_timezone
        .as_deref()
        .unwrap_or("UTC");

    let (start, end) = if event.all_day {
        // All-day events use `date` not `dateTime`; strip the time portion.
        let start_date = event.start_at.get(..10).unwrap_or(&event.start_at);
        let end_date = event.end_at.get(..10).unwrap_or(&event.end_at);
        (
            json!({ "date": start_date }),
            json!({ "date": end_date }),
        )
    } else {
        (
            json!({ "dateTime": event.start_at, "timeZone": tz }),
            json!({
                "dateTime": event.end_at,
                "timeZone": event.end_timezone.as_deref().unwrap_or(tz)
            }),
        )
    };

    let mut body = json!({
        "summary": event.title,
        "start": start,
        "end": end,
        "calendarId": calendar_id,
    });

    if let Some(desc) = &event.description {
        body["description"] = json!(desc);
    }
    if let Some(loc) = &event.location {
        body["location"] = json!(loc);
    }

    body
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_gcal_event_args_timed_event() {
        let event = EventSummary {
            id: "ev1".to_string(),
            title: "Team standup".to_string(),
            all_day: false,
            start_at: "2024-03-01T09:00:00.000Z".to_string(),
            start_timezone: Some("Asia/Taipei".to_string()),
            end_at: "2024-03-01T09:30:00.000Z".to_string(),
            end_timezone: Some("Asia/Taipei".to_string()),
            description: Some("Daily sync".to_string()),
            location: None,
        };

        let args = build_gcal_event_arguments(&event, "primary");

        assert_eq!(args["summary"], "Team standup");
        assert_eq!(args["calendarId"], "primary");
        assert_eq!(args["start"]["dateTime"], "2024-03-01T09:00:00.000Z");
        assert_eq!(args["start"]["timeZone"], "Asia/Taipei");
        assert_eq!(args["end"]["dateTime"], "2024-03-01T09:30:00.000Z");
        assert_eq!(args["description"], "Daily sync");
        assert!(args.get("location").map(|v| v.is_null()).unwrap_or(true));
    }

    #[test]
    fn build_gcal_event_args_all_day() {
        let event = EventSummary {
            id: "ev2".to_string(),
            title: "Holiday".to_string(),
            all_day: true,
            start_at: "2024-03-04T00:00:00.000Z".to_string(),
            start_timezone: None,
            end_at: "2024-03-04T00:00:00.000Z".to_string(),
            end_timezone: None,
            description: None,
            location: None,
        };

        let args = build_gcal_event_arguments(&event, "primary");

        assert_eq!(args["summary"], "Holiday");
        assert_eq!(args["start"]["date"], "2024-03-04");
        assert_eq!(args["end"]["date"], "2024-03-04");
        assert!(args["start"].get("dateTime").is_none());
    }

    #[test]
    fn build_gcal_event_args_fallback_timezone() {
        let event = EventSummary {
            id: "ev3".to_string(),
            title: "No-tz event".to_string(),
            all_day: false,
            start_at: "2024-03-01T10:00:00.000Z".to_string(),
            start_timezone: None,
            end_at: "2024-03-01T11:00:00.000Z".to_string(),
            end_timezone: None,
            description: None,
            location: None,
        };

        let args = build_gcal_event_arguments(&event, "primary");
        assert_eq!(args["start"]["timeZone"], "UTC");
        assert_eq!(args["end"]["timeZone"], "UTC");
    }
}
