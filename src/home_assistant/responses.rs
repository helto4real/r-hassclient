use serde_json::Value;
use serde::{Deserialize, Serialize};
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
#[derive(Serialize, Deserialize)]
pub struct WsResult {
    pub(crate) id: u64,
    pub(crate) success: bool,
    pub(crate) result: Value,
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct WsEvent {
    pub id: u64,
    pub event: HaEvent,
}
