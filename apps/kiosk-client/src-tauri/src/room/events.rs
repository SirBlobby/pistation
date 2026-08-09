use serde::Serialize;

pub const STATUS_EVENT: &str = "room://status";
pub const DATA_EVENT: &str = "room://data";
pub const VIDEO_EVENT: &str = "room://video";
pub const PARTICIPANTS_EVENT: &str = "room://participants";

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticipantView {
    pub identity: String,
    pub display_name: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticipantsPayload {
    pub participants: Vec<ParticipantView>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusPayload {
    pub status: &'static str,
    pub detail: Option<String>,
}

impl StatusPayload {
    pub fn new(status: &'static str) -> Self {
        Self {
            status,
            detail: None,
        }
    }

    pub fn with_detail(status: &'static str, detail: String) -> Self {
        Self {
            status,
            detail: Some(detail),
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataPayload {
    pub envelope: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoPayload {
    pub active: bool,
    pub width: u32,
    pub height: u32,
}
