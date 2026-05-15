//! Thin HTTP client for the TimeTree REST API v1.
//!
//! Base URL: `https://timetreeapp.com/api/v1`
//! Auth: `Authorization: Bearer <personal_access_token>`
//!
//! Docs: <https://developers.timetreeapp.com/en/docs/api>

use std::time::Duration;

use anyhow::{Context, Result};

use super::types::{TimeTreeCalendarsResponse, TimeTreeEventsResponse};

const BASE_URL: &str = "https://timetreeapp.com/api/v1";
const REQUEST_TIMEOUT_SECS: u64 = 30;

pub struct TimeTreeClient {
    access_token: String,
    http: reqwest::Client,
}

impl TimeTreeClient {
    pub fn new(access_token: impl Into<String>) -> Result<Self> {
        let http = reqwest::Client::builder()
            .use_rustls_tls()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .user_agent("openhuman/1.0")
            .build()
            .context("[timetree] failed to build HTTP client")?;
        Ok(Self {
            access_token: access_token.into(),
            http,
        })
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.access_token)
    }

    /// `GET /calendars` — list all calendars the token can access.
    pub async fn list_calendars(&self) -> Result<TimeTreeCalendarsResponse> {
        let url = format!("{BASE_URL}/calendars");
        tracing::debug!("[timetree] GET /calendars");
        let resp = self
            .http
            .get(&url)
            .header("Authorization", self.auth_header())
            .header("Accept", "application/vnd.timetree.v1+json")
            .send()
            .await
            .context("[timetree] list_calendars request failed")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("[timetree] list_calendars HTTP {status}: {body}");
        }

        resp.json::<TimeTreeCalendarsResponse>()
            .await
            .context("[timetree] list_calendars: failed to parse response")
    }

    /// `GET /calendars/{id}/upcoming_events` — list upcoming events.
    ///
    /// `days` is the lookahead window (1–7 for the free plan, higher for paid).
    /// `timezone` is an IANA timezone name, e.g. `"Asia/Taipei"`.
    pub async fn list_upcoming_events(
        &self,
        calendar_id: &str,
        days: u32,
        timezone: &str,
    ) -> Result<TimeTreeEventsResponse> {
        let url = format!("{BASE_URL}/calendars/{calendar_id}/upcoming_events");
        tracing::debug!(
            calendar_id = %calendar_id,
            days = days,
            timezone = %timezone,
            "[timetree] GET /calendars/{}/upcoming_events",
            calendar_id
        );
        let resp = self
            .http
            .get(&url)
            .header("Authorization", self.auth_header())
            .header("Accept", "application/vnd.timetree.v1+json")
            .query(&[("days", days.to_string()), ("timezone", timezone.to_string())])
            .send()
            .await
            .context("[timetree] list_upcoming_events request failed")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("[timetree] list_upcoming_events HTTP {status}: {body}");
        }

        resp.json::<TimeTreeEventsResponse>()
            .await
            .context("[timetree] list_upcoming_events: failed to parse response")
    }
}
