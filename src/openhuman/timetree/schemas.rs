//! Controller schema definitions and registered handlers for the `timetree` domain.

use serde_json::{Map, Value};

use crate::core::all::{ControllerFuture, RegisteredController};
use crate::core::{ControllerSchema, FieldSchema, TypeSchema};

type SchemaBuilder = fn() -> ControllerSchema;
type ControllerHandler = fn(Map<String, Value>) -> ControllerFuture;

struct TimeTreeControllerDef {
    function: &'static str,
    schema: SchemaBuilder,
    handler: ControllerHandler,
}

const TIMETREE_CONTROLLER_DEFS: &[TimeTreeControllerDef] = &[
    TimeTreeControllerDef {
        function: "list_calendars",
        schema: schema_list_calendars,
        handler: handle_list_calendars_wrap,
    },
    TimeTreeControllerDef {
        function: "list_events",
        schema: schema_list_events,
        handler: handle_list_events_wrap,
    },
    TimeTreeControllerDef {
        function: "sync_to_google_calendar",
        schema: schema_sync_to_google_calendar,
        handler: handle_sync_to_google_calendar_wrap,
    },
];

pub fn all_controller_schemas() -> Vec<ControllerSchema> {
    TIMETREE_CONTROLLER_DEFS
        .iter()
        .map(|def| (def.schema)())
        .collect()
}

pub fn all_registered_controllers() -> Vec<RegisteredController> {
    TIMETREE_CONTROLLER_DEFS
        .iter()
        .map(|def| RegisteredController {
            schema: (def.schema)(),
            handler: def.handler,
        })
        .collect()
}

pub fn schemas(function: &str) -> ControllerSchema {
    TIMETREE_CONTROLLER_DEFS
        .iter()
        .find(|def| def.function == function)
        .map(|def| (def.schema)())
        .unwrap_or_else(schema_unknown)
}

// ── Schema definitions ───────────────────────────────────────────────────────

fn schema_list_calendars() -> ControllerSchema {
    ControllerSchema {
        namespace: "timetree",
        function: "list_calendars",
        description: "List all calendars the user can access in TimeTree using their \
                      personal access token. Returns id, name, description, and color \
                      for each calendar.",
        inputs: vec![FieldSchema {
            name: "access_token",
            ty: TypeSchema::String,
            comment: "TimeTree personal access token. Generate one at \
                      https://timetreeapp.com/personal_access_tokens",
            required: true,
        }],
        outputs: vec![FieldSchema {
            name: "data",
            ty: TypeSchema::Array(Box::new(TypeSchema::Json)),
            comment: "Array of calendar objects: {id, name, description?, color?}",
            required: true,
        }],
    }
}

fn schema_list_events() -> ControllerSchema {
    ControllerSchema {
        namespace: "timetree",
        function: "list_events",
        description: "List upcoming events from a specific TimeTree calendar. \
                      Returns up to `days` days of future events.",
        inputs: vec![
            FieldSchema {
                name: "access_token",
                ty: TypeSchema::String,
                comment: "TimeTree personal access token.",
                required: true,
            },
            FieldSchema {
                name: "calendar_id",
                ty: TypeSchema::String,
                comment: "TimeTree calendar ID (from timetree_list_calendars).",
                required: true,
            },
            FieldSchema {
                name: "days",
                ty: TypeSchema::U64,
                comment: "Number of days ahead to fetch (default 30). Free plans are limited to 7.",
                required: false,
            },
            FieldSchema {
                name: "timezone",
                ty: TypeSchema::String,
                comment: "IANA timezone name, e.g. \"Asia/Taipei\" (default \"UTC\").",
                required: false,
            },
        ],
        outputs: vec![FieldSchema {
            name: "data",
            ty: TypeSchema::Array(Box::new(TypeSchema::Json)),
            comment: "Array of event objects: {id, title, all_day, start_at, end_at, \
                      start_timezone?, end_timezone?, description?, location?}",
            required: true,
        }],
    }
}

fn schema_sync_to_google_calendar() -> ControllerSchema {
    ControllerSchema {
        namespace: "timetree",
        function: "sync_to_google_calendar",
        description: "Fetch upcoming events from a TimeTree calendar and create them \
                      in a Google Calendar via the connected Composio Google Calendar \
                      account. Returns synced_count, skipped_count, and any errors.",
        inputs: vec![
            FieldSchema {
                name: "access_token",
                ty: TypeSchema::String,
                comment: "TimeTree personal access token.",
                required: true,
            },
            FieldSchema {
                name: "calendar_id",
                ty: TypeSchema::String,
                comment: "TimeTree calendar ID to read events from.",
                required: true,
            },
            FieldSchema {
                name: "days",
                ty: TypeSchema::U64,
                comment: "Lookahead window in days (default 30). Free plans: max 7.",
                required: false,
            },
            FieldSchema {
                name: "timezone",
                ty: TypeSchema::String,
                comment: "IANA timezone for event expansion (default \"UTC\").",
                required: false,
            },
            FieldSchema {
                name: "google_calendar_id",
                ty: TypeSchema::String,
                comment: "Target Google Calendar ID. Use \"primary\" for the default \
                          calendar (default \"primary\").",
                required: false,
            },
        ],
        outputs: vec![
            FieldSchema {
                name: "synced_count",
                ty: TypeSchema::U64,
                comment: "Number of events successfully created in Google Calendar.",
                required: true,
            },
            FieldSchema {
                name: "skipped_count",
                ty: TypeSchema::U64,
                comment: "Number of events that failed to sync.",
                required: true,
            },
            FieldSchema {
                name: "errors",
                ty: TypeSchema::Array(Box::new(TypeSchema::String)),
                comment: "Error messages for skipped events.",
                required: true,
            },
        ],
    }
}

fn schema_unknown() -> ControllerSchema {
    ControllerSchema {
        namespace: "timetree",
        function: "unknown",
        description: "Unknown timetree controller function.",
        inputs: vec![FieldSchema {
            name: "function",
            ty: TypeSchema::String,
            comment: "Unknown function requested.",
            required: true,
        }],
        outputs: vec![FieldSchema {
            name: "error",
            ty: TypeSchema::String,
            comment: "Lookup error details.",
            required: true,
        }],
    }
}

// ── Handler wrappers ─────────────────────────────────────────────────────────

fn handle_list_calendars_wrap(params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move { super::rpc::handle_list_calendars(params).await })
}

fn handle_list_events_wrap(params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move { super::rpc::handle_list_events(params).await })
}

fn handle_sync_to_google_calendar_wrap(params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move { super::rpc::handle_sync_to_google_calendar(params).await })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_schemas_have_correct_namespace() {
        for s in all_controller_schemas() {
            assert_eq!(s.namespace, "timetree", "schema {} has wrong namespace", s.function);
        }
    }

    #[test]
    fn all_registered_controllers_match_schema_count() {
        let schemas = all_controller_schemas();
        let handlers = all_registered_controllers();
        assert_eq!(schemas.len(), handlers.len());
        assert_eq!(schemas.len(), TIMETREE_CONTROLLER_DEFS.len());
    }

    #[test]
    fn list_calendars_requires_access_token() {
        let s = schema_list_calendars();
        assert_eq!(s.function, "list_calendars");
        let req: Vec<_> = s.inputs.iter().filter(|f| f.required).map(|f| f.name).collect();
        assert_eq!(req, vec!["access_token"]);
    }

    #[test]
    fn list_events_required_inputs() {
        let s = schema_list_events();
        let req: Vec<_> = s.inputs.iter().filter(|f| f.required).map(|f| f.name).collect();
        assert_eq!(req, vec!["access_token", "calendar_id"]);
    }

    #[test]
    fn sync_required_inputs() {
        let s = schema_sync_to_google_calendar();
        let req: Vec<_> = s.inputs.iter().filter(|f| f.required).map(|f| f.name).collect();
        assert_eq!(req, vec!["access_token", "calendar_id"]);
    }

    #[test]
    fn lookup_unknown_function() {
        let s = schemas("does_not_exist");
        assert_eq!(s.function, "unknown");
    }

    #[test]
    fn sync_outputs_include_synced_count() {
        let s = schema_sync_to_google_calendar();
        assert!(s.outputs.iter().any(|f| f.name == "synced_count" && f.required));
        assert!(s.outputs.iter().any(|f| f.name == "skipped_count" && f.required));
        assert!(s.outputs.iter().any(|f| f.name == "errors" && f.required));
    }
}
