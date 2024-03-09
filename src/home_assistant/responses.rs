use serde_json::Value;
use serde::Deserialize;
use crate::home_assistant::model::*;

#[derive(Debug)]
#[derive(Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub(crate) enum Response {
    AuthRequired(AuthRequired),
    AuthOk(AuthOk),
    AuthInvalid(AuthInvalid),
    Event(WsEvent),
    Result(WsResult),
    Pong(WSPong),
    #[serde(other)]
    Unknown
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) struct AuthRequired {
    pub(crate) ha_version: String,
}

// this is received when the service successfully autheticate
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) struct AuthOk {
    pub(crate) ha_version: String,
}

// this is received if the authetication failed
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) struct AuthInvalid {
    pub(crate) message: String,
}


#[derive(Debug)]
#[derive(Deserialize)]
pub struct WsResult {
    pub(crate) id: u64,
    pub(crate) success: bool,
    pub(crate) result: Value,
    pub(crate) error: Option<ErrorCode>,
}

#[derive(Debug)]
#[derive(Deserialize)]
pub struct WsEvent {
    pub id: u64,
    pub event: HaEvent,
}
#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct ErrorCode {
    pub(crate) code: String,
    pub(crate) message: String,
}

// this is received as a response to a ping request
#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct WSPong {
    pub(crate) id: u64,
}
