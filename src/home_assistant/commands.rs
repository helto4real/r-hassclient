use serde_json::{Value, json};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Serialize, PartialEq)]
pub(crate) enum HaCommand {
    Auth(Auth),
}

impl HaCommand {
pub(crate) fn to_tungstenite_message(self) -> Message {
        match self {
            Self::Auth(auth) => {
                let cmd_str = serde_json::to_string(&auth).unwrap();
                Message::Text(cmd_str)
            }
            // Self::Ping(ping) => {
            //     let cmd_str = serde_json::to_string(&ping).unwrap();
            //     TungsteniteMessage::Text(cmd_str)
            // }
            // Self::SubscribeEvent(subscribe) => {
            //     let cmd_str = serde_json::to_string(&subscribe).unwrap();
            //     TungsteniteMessage::Text(cmd_str)
            // }
            // Self::Unsubscribe(unsubscribe) => {
            //     let cmd_str = serde_json::to_string(&unsubscribe).unwrap();
            //     TungsteniteMessage::Text(cmd_str)
            // }
            // Self::GetConfig(getconfig) => {
            //     let cmd_str = serde_json::to_string(&getconfig).unwrap();
            //     TungsteniteMessage::Text(cmd_str)
            // }
            // Self::GetStates(getstates) => {
            //     let cmd_str = serde_json::to_string(&getstates).unwrap();
            //     TungsteniteMessage::Text(cmd_str)
            // }
            // Self::GetServices(getservices) => {
            //     let cmd_str = serde_json::to_string(&getservices).unwrap();
            //     TungsteniteMessage::Text(cmd_str)
            // }
            // Self::GetPanels(getpanels) => {
            //     let cmd_str = serde_json::to_string(&getpanels).unwrap();
            //     TungsteniteMessage::Text(cmd_str)
            // }
            // Self::CallService(callservice) => {
            //     let cmd_str = serde_json::to_string(&callservice).unwrap();
            //     TungsteniteMessage::Text(cmd_str)
            // }
            // Self::Close => todo!(),
        }
    }

}
#[derive(Debug, Serialize, PartialEq)]
pub(crate) struct Auth {
    #[serde(rename = "type")]
    pub(crate) msg_type: String,
    pub(crate) access_token: String,
}
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub(crate) struct SubscribeToEventsCommand {
    #[serde(rename = "type")]
    typ: String,
    event_type: Option<String>,
}

pub(crate) fn new_subscribe_to_events_command(id: u32, event_type: &str) -> Value {
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
