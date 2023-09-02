use std::iter::Map;
use serde_json::{Value, json};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::home_assistant::model::*;

#[derive(Debug)]
#[derive(Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Response {
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
pub struct AuthRequired {
    pub ha_version: String,
}

// this is received when the service successfully autheticate
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct AuthOk {
    pub(crate) ha_version: String,
}

// this is received if the authetication failed
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct AuthInvalid {
    pub(crate) message: String,
}


#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct WsResult {
    success: bool,
    result: Value,
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct WsEvent {
    pub id: u64,
    pub event: HaEvent,
}