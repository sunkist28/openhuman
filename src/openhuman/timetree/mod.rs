//! TimeTree → Google Calendar sync domain.
//!
//! Provides three RPC controllers:
//!
//! - `timetree.list_calendars`            → `openhuman.timetree_list_calendars`
//! - `timetree.list_events`               → `openhuman.timetree_list_events`
//! - `timetree.sync_to_google_calendar`   → `openhuman.timetree_sync_to_google_calendar`
//!
//! ## Authentication
//!
//! TimeTree uses a **personal access token** generated at
//! <https://timetreeapp.com/personal_access_tokens>. The token is passed as
//! the `access_token` parameter on every call — the core does not persist it.
//!
//! Google Calendar writes go through the Composio `GOOGLECALENDAR_CREATE_EVENT`
//! tool, which requires the user to have authorized Google Calendar in Composio
//! first (`openhuman.composio_authorize` with `toolkit = "googlecalendar"`).
//!
//! ## Module layout
//!
//! - [`types`]   — TimeTree API response shapes and flattened RPC types
//! - [`client`]  — Thin `reqwest`-based HTTP client for the TimeTree REST API
//! - [`ops`]     — Domain operations (list, sync)
//! - [`rpc`]     — Async JSON-RPC handler functions
//! - [`schemas`] — Controller schema definitions and registered handler wrappers

pub mod client;
pub mod ops;
pub mod rpc;
pub mod schemas;
pub mod types;

pub use schemas::{
    all_controller_schemas as all_timetree_controller_schemas,
    all_registered_controllers as all_timetree_registered_controllers,
};
