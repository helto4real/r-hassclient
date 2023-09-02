use std::any::Any;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;


#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[serde(tag = "event_type", content = "data")]
pub enum HaEvent {
    #[serde(rename = "state_changed")]
    StateChangedEvent(StateChangedEvent),
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]

pub struct StateChangedEvent {
    entity_id: String,
    new_state: Option<HaState>,
    old_state: Option<HaState>,
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct HaState {
    entity_id: String,
    attributes: Option<HashMap<String, Value>>,
    state: String,
}

// #[derive(Debug)]
// #[derive(Serialize, Deserialize)]
// pub struct HaResult {
//     success: bool,
//     error: Option<HaError>,
//    
// }

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct HaError {
    code: String,
    message: String,
}