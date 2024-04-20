use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

use std::collections::HashMap;

use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
// #[serde(tag = "event_type", content = "data")]
pub enum HaEventData {
    // #[serde(rename = "state_changed")]
    StateChangedEvent(StateChangedEvent),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct HaEvent {
    // #[serde(deserialize_with = "deserialize_my_data")]
    pub data: Value,
    pub event_type: String,
    pub time_fired: String,
    pub origin: String,
    // pub context: Context,
}

impl HaEvent {
    pub fn get_event_data(&self) -> Result<HaEventData, serde_json::Error> {
        match self.event_type.as_str() {
            "state_changed" => {
                let data = StateChangedEvent::deserialize(self.data.clone())
                    .map_err(serde::de::Error::custom)?;
                Ok(HaEventData::StateChangedEvent(data))
            }
            _ => Err(serde::de::Error::custom("unknown type")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]

pub struct StateChangedEvent {
    pub entity_id: String,
    pub new_state: Option<HaState>,
    pub old_state: Option<HaState>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HaState {
    pub entity_id: String,
    pub attributes: Option<HashMap<String, Value>>,
    pub state: String,
}

impl fmt::Display for StateChangedEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "HassEvent {{")?;
        // write!(f, "  event_type: {},\n", self.event_type)?;
        // write!(f, "  data: {{\n")?;
        writeln!(f, "    entity_id: {:?},", self.entity_id)?;
        writeln!(f, "    new_state: {:?},", self.new_state)?;
        writeln!(f, "    old_state: {:?},", self.old_state)?;
        writeln!(f, "  }},")?;
        // write!(f, "  origin: {},\n", self.origin)?;
        // write!(f, "  time_fired: {},\n", self.time_fired)?;
        // write!(f, "  context: {:?},\n", self.context)?;
        write!(f, "}}")?;
        Ok(())
    }
}
