use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;


#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[serde(tag = "event_type", content = "data")]
pub enum HaEvent {
    #[serde(rename = "state_changed")]
    StateChangedEvent(StateChangedEvent),
}

#[derive(Debug,Clone)]
#[derive(Serialize, Deserialize)]

pub struct StateChangedEvent {
    pub entity_id: String,
    pub new_state: Option<HaState>,
    pub old_state: Option<HaState>,
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct HaState {
    pub entity_id: String,
    pub attributes: Option<HashMap<String, Value>>,
    pub state: String,
}

// #[derive(Debug)]
// #[derive(Serialize, Deserialize)]
// pub struct HaResult {
//     success: bool,
//     error: Option<HaError>,
//    
// }

#[derive(Debug,Clone)]
#[derive(Serialize, Deserialize)]
pub struct HaError {
    pub code: String,
    pub message: String,
}
