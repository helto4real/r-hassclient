use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;

// Todo: these warnings is probably due to bad visibility that I do not really
// understand yet :)
#[derive(Serialize, PartialEq)]
pub(crate) enum HaCommand {
    AuthInfo(Auth),
    Ping(Ask),
    SubscribeEvent(Subscribe),
}

impl HaCommand {
    pub(crate) fn to_tungstenite_message(&self) -> Message {
        match self {
            Self::AuthInfo(auth) => {
                let cmd_str = serde_json::to_string(&auth).unwrap();
                Message::Text(cmd_str)
            }
            Self::Ping(ping) => {
                let cmd_str = serde_json::to_string(&ping).unwrap();
                Message::Text(cmd_str)
            }
            Self::SubscribeEvent(subscribe) => {
                let cmd_str = serde_json::to_string(&subscribe).unwrap();
                Message::Text(cmd_str)
            }
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
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct SubscribeToEventsCommand {
    #[serde(rename = "type")]
    typ: String,
    event_type: Option<String>,
}
//used to fetch from server
#[derive(Debug, Serialize, PartialEq)]
pub(crate) struct Ask {
    pub(crate) id: Option<u64>,
    #[serde(rename = "type")]
    pub(crate) msg_type: String,
}
//used for Event subscribtion
#[derive(Debug, Serialize, PartialEq)]
pub(crate) struct Subscribe {
    pub(crate) id: Option<u64>,
    #[serde(rename = "type")]
    pub(crate) msg_type: String,
    pub(crate) event_type: String,
}

