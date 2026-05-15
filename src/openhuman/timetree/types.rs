//! TimeTree API request/response types.
//!
//! Mirrors the JSON:API shapes returned by `https://timetreeapp.com/api/v1`.

use serde::{Deserialize, Serialize};

// ── Calendar ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct TimeTreeCalendarAttributes {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TimeTreeCalendarResource {
    pub id: String,
    pub attributes: TimeTreeCalendarAttributes,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TimeTreeCalendarsResponse {
    pub data: Vec<TimeTreeCalendarResource>,
}

/// Flattened calendar record surfaced over RPC.
#[derive(Debug, Clone, Serialize)]
pub struct CalendarSummary {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

impl From<TimeTreeCalendarResource> for CalendarSummary {
    fn from(r: TimeTreeCalendarResource) -> Self {
        Self {
            id: r.id,
            name: r.attributes.name,
            description: r.attributes.description,
            color: r.attributes.color,
        }
    }
}

// ── Event ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct TimeTreeEventAttributes {
    pub title: String,
    #[serde(default)]
    pub all_day: bool,
    /// ISO 8601 datetime string, e.g. "2024-03-01T09:00:00.000Z"
    pub start_at: String,
    #[serde(default)]
    pub start_timezone: Option<String>,
    pub end_at: String,
    #[serde(default)]
    pub end_timezone: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TimeTreeEventResource {
    pub id: String,
    pub attributes: TimeTreeEventAttributes,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TimeTreeEventsResponse {
    pub data: Vec<TimeTreeEventResource>,
}

/// Flattened event record surfaced over RPC and used for Google Calendar sync.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSummary {
    pub id: String,
    pub title: String,
    pub all_day: bool,
    pub start_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_timezone: Option<String>,
    pub end_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_timezone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

impl From<TimeTreeEventResource> for EventSummary {
    fn from(r: TimeTreeEventResource) -> Self {
        Self {
            id: r.id,
            title: r.attributes.title,
            all_day: r.attributes.all_day,
            start_at: r.attributes.start_at,
            start_timezone: r.attributes.start_timezone,
            end_at: r.attributes.end_at,
            end_timezone: r.attributes.end_timezone,
            description: r.attributes.description.filter(|s| !s.is_empty()),
            location: r.attributes.location.filter(|s| !s.is_empty()),
        }
    }
}

// ── Sync result ──────────────────────────────────────────────────────────────

/// Summary returned after a `timetree_sync_to_google_calendar` call.
#[derive(Debug, Clone, Serialize)]
pub struct SyncResult {
    pub synced_count: usize,
    pub skipped_count: usize,
    pub errors: Vec<String>,
}
