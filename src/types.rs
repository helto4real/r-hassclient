use std::fmt;
use simple_error::SimpleError;
use tokio_tungstenite::{connect_async, MaybeTlsStream, tungstenite::protocol::Message, WebSocketStream};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Error as TungsteniteError;

pub (crate) type WebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub type HassResult<T> = std::result::Result<T, HassError>;

#[derive(Debug)]
pub enum HassError {
    /// Returned when the connection to gateway has failed
    CantConnectToHomeAssistant,

    /// Returned when it is unable to authenticate
    AuthenticationFailed(String),

    /// Returned when serde was unable to deserialize the values
    UnableToDeserialize(serde_json::error::Error),

    /// Tungstenite error
    TungsteniteError(TungsteniteError),
    
    /// Returned when unable to parse the websocket server address
    WrongAddressProvided(url::ParseError),
    
    /// Returned for errors which do not fit any of the above criterias
    Generic(String),
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
            Self::UnableToDeserialize(e) => {
                write!(f, "Unable to deserialize the received value: {}", e)
            }
            Self::TungsteniteError(e) => write!(f, "Tungstenite Error: {}", e),
            // Self::ChannelSend(e) => write!(f, "Channel Send Error: {}", e),
            // Self::UnknownPayloadReceived => write!(f, "The received payload is unknown"),
            // Self::ReponseError(e) => write!(
            //     f,
            //     "The error code:{} with the error message: {}",
            //     e.error.as_ref().unwrap().code,
            //     e.error.as_ref().unwrap().message
            // ),
            Self::Generic(detail) => write!(f, "Generic Error: {}", detail),
        }
    }
}

impl From<SimpleError> for HassError {
    fn from(error: SimpleError) -> Self {
        HassError::Generic(error.to_string())
    }
}


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