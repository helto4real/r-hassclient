use serde::Serialize;
use serde_json::Value;
use tokio_tungstenite::tungstenite::Message;

// Todo: these warnings is probably due to bad visibility that I do not really
// understand yet :)
#[derive(Debug)]
pub(crate) enum HaCommand {
    AuthInfo(Auth),
    Ping(Ask),
    SubscribeEvent(Subscribe),
    CallService(CallService),
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
            Self::CallService(callservice) => {
                let cmd_str = serde_json::to_string(&callservice).unwrap();
                Message::Text(cmd_str)
            }
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
#[derive(Debug, Serialize, PartialEq)]
pub(crate) struct CallService {
    pub(crate) id: Option<u64>,
    #[serde(rename = "type")]
    pub(crate) msg_type: String,
    pub(crate) domain: String,
    pub(crate) service: String,
    pub(crate) service_data: Option<Value>,
}
