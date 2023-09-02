use serde_json::{Value, json};
use serde::{Deserialize, Serialize};
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct SubscribeToEventsCommand {
    #[serde(rename = "type")]
    typ: String,
    event_type: Option<String>,
}

pub fn new_subscribe_to_events_command(id: u32, event_type: &str) -> Value {
    return json!({
        "id": id,
        "type": "subscribe_events",
        "event_type": event_type
    });
}
// 
// #[derive(Serialize, Deserialize)]
// struct Context {
//     pub id: String,
//     pub parent_id: Option<_>,
//     pub user_id: String,
// }
// 
// #[derive(Serialize, Deserialize)]
// struct Attributes {
//     pub editable: bool,
//     pub friendly_name: String,
// }
// 
// #[derive(Serialize, Deserialize)]
// struct Struct {
//     pub entity_id: String,
//     pub state: String,
//     pub attributes: Attributes,
//     pub last_changed: String,
//     pub last_updated: String,
//     pub context: Context,
// }
// 
// #[derive(Serialize, Deserialize)]
// struct Data {
//     pub entity_id: String,
//     pub old_state: Struct,
//     pub new_state: Struct,
// }
// 
// #[derive(Serialize, Deserialize)]
// struct Result {
//     pub id: i64,
//     #[serde(rename = "type")]
//     pub r#type: String,
//     pub success: bool,
//     pub result: Option<_>,
// }
// 
// #[derive(Serialize, Deserialize)]
// struct Event {
//     pub event_type: String,
//     pub data: Data,
//     pub origin: String,
//     pub time_fired: String,
//     pub context: Context,
// }
// 
// #[derive(Serialize, Deserialize)]
// struct Root {
//     pub id: i64,
//     #[serde(rename = "type")]
//     pub r#type: String,
//     pub event: Event,
// }