use simple_error::SimpleError;
use std::fmt;
use tokio_tungstenite::tungstenite::Error as TungsteniteError;

use crate::home_assistant::responses::WsResult;

//pub (crate) type WebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub type HassResult<T> = std::result::Result<T, HassError>;

#[derive(Debug)]
pub enum HassError {
    /// Returned when the connection to gateway has failed
    CantConnectToHomeAssistant,

    /// Returned when it is unable to authenticate
    AuthenticationFailed(String),

    /// Returned when serde was unable to deserialize the values
    UnableToDeserialize(serde_json::error::Error),

    SendError,
    /// Tungstenite error
    TungsteniteError(TungsteniteError),

    /// Returned when unable to parse the websocket server address
    WrongAddressProvided(url::ParseError),

    // Return if the underlying websocket connection somehow faults
    ConnectionError,

    /// Returned for errors which do not fit any of the above criterias
    GenericError(String),
    UnknownPayloadReceived,
    ResponseError(WsResult),
}

impl std::error::Error for HassError {}

impl fmt::Display for HassError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::CantConnectToHomeAssistant => write!(f, "Cannot connect to Home Assistant"),
            // Self::ConnectionClosed => write!(f, "Connection closed unexpectedly"),
            Self::AuthenticationFailed(e) => write!(f, "Authentication has failed: {}", e),
            Self::WrongAddressProvided(e) => {
                write!(f, "Could not parse the provided address: {}", e)
            }
            Self::ConnectionError => write!(f, "Connection closed unexpectedly"),
            Self::UnableToDeserialize(e) => {
                write!(f, "Unable to deserialize the received value: {}", e)
            }
            Self::TungsteniteError(e) => write!(f, "Tungstenite Error: {}", e),
            Self::SendError => write!(f, "Send Error"),
            // Self::ChannelSend(e) => write!(f, "Channel Send Error: {}", e),
            Self::UnknownPayloadReceived => write!(f, "The received payload is unknown"),
            Self::ResponseError(e) => write!(
                f,
                "The error code:{} with the error message: {}",
                e.error.as_ref().unwrap().code,
                e.error.as_ref().unwrap().message
            ),
            Self::GenericError(detail) => write!(f, "Generic Error: {}", detail),
        }
    }
}
impl From<SimpleError> for HassError {
    fn from(error: SimpleError) -> Self {
        HassError::GenericError(error.to_string())
    }
}

// impl From<tokio::sync::mpsc::error::SendError> for HassError {
//     fn from(error: tokio::sync::mpsc:error::SendError) -> Self {
//         HassError::SendError(error)
//     }
// }
impl From<serde_json::error::Error> for HassError {
    fn from(error: serde_json::error::Error) -> Self {
        HassError::UnableToDeserialize(error)
    }
}

impl From<url::ParseError> for HassError {
    fn from(error: url::ParseError) -> Self {
        HassError::WrongAddressProvided(error)
    }
}

impl From<TungsteniteError> for HassError {
    fn from(error: TungsteniteError) -> Self {
        // let e = match error {
        //     tungstenite::error::Error::ConnectionClosed => {
        //         tungstenite::error::Error::ConnectionClosed
        //     }
        //     tungstenite::error::Error::AlreadyClosed => tungstenite::error::Error::AlreadyClosed,
        //     _ => return HassError::Generic(format!("Error from ws {}", error)),
        // };
        HassError::TungsteniteError(error)
    }
}
