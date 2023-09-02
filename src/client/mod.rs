mod event_stream;

use crate::{
    home_assistant::{commands::*, responses::Response},
    types::{HassResult, WebSocket},
};
use colored::Colorize;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use serde_json::{from_value, Value};
use std::error::Error;
use std::path::is_separator;
use std::str::Matches;
use std::thread::sleep;
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};
use tokio_util::sync::CancellationToken;

use self::event_stream::HaEventStream;

pub struct HaClient {
    // url: String,
    on_connected_cb: Option<Box<dyn FnMut()>>,
}

#[derive(Default)]
pub struct HaClientBuilder {
    on_connected_cb: Option<Box<dyn FnMut()>>,
}

impl HaClientBuilder {
    pub fn new() -> HaClientBuilder {
        // Set the minimally required fields of Foo.
        HaClientBuilder {
            on_connected_cb: None,
        }
    }
    pub fn on_connected(mut self, func: Box<dyn FnMut()>) -> HaClientBuilder {
        self.on_connected_cb = Some(func);
        self
    }
    pub fn build(self) -> HaClient {
        HaClient {
            on_connected_cb: self.on_connected_cb,
        }
    }
}

async fn get_response_json(
    reader: &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
) -> HassResult<Response> {
    let msg = reader
        .next()
        .await
        .ok_or(simple_error!("Empty result from websocket reader."))?;
    let response = serde_json::from_slice(&msg?.into_data())?;
    return Ok(response);
}

// async fn read_message_json(
//     reader: &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
// ) -> Result<(Value, String), Box<dyn Error>> {
//     let msg = reader
//         .next()
//         .await
//         .ok_or(simple_error!("Empty result from websocket reader."))?;
//     let msg = msg?.into_text()?;
//     let msg_json: Value = serde_json::from_str(&msg)?;
//     let msg_type = msg_json["type"]
//         .as_str()
//         .ok_or(simple_error!("Type could not be read from message."))?
//         .to_string();
//     return Ok((msg_json, msg_type));
// }
//
// async fn read_message_of_type_json(
//     reader: &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
//     msg_type: &str,
// ) -> Result<Value, Box<dyn Error>> {
//     let (msg, actual_msg_type) = read_message_json(reader).await?;
//     if actual_msg_type != msg_type {
//         bail!(format!(
//             "Expected '{}' message, got '{}'",
//             msg_type, actual_msg_type
//         ));
//     }
//     return Ok(msg);
// }

impl HaClient {
    pub fn builder() -> HaClientBuilder {
        HaClientBuilder::default()
    }
    pub async fn connect_ha_async(&mut self, url: &str, token: &str) -> HassResult<HaConnection> {
        let uri = url::Url::parse(&url)?;
        let ws_stream = connect_ha_async(uri).await?;
        let (mut write, mut read): (
            SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
            SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
        ) = ws_stream.split();

        authorize_home_assistant(token, &mut read, &mut write).await?;

        // channel to handle all new messages from the websocket
        let (tx_receiver, rx_receiver): (Sender<String>, Receiver<String>) = mpsc::channel(100);
        // channel to handle all messages that should be sent on the websocket
        let (tx_sender, rx_sender): (Sender<String>, Receiver<String>) = mpsc::channel(100);
        // let recv_config = ReceiverConfig::new(String::from(token));

        let cloned_sender = tx_sender.clone();
        let ct = CancellationToken::new();
        let ct_msg_hndl = ct.clone();
        let handle_message_handler = tokio::spawn(async move {
            select! {
                _ = ct_msg_hndl.cancelled() => {
                    // The token was cancelled

                }
                _ = handle_messages(rx_receiver, cloned_sender) => {
                }

            }
            println!("Exiting handle_message_handler");
        });

        let ct_msg_disp = ct.clone();
        let ha_msg_dispatcher_handler = tokio::spawn(async move {
            select! {
                _ = ct_msg_disp.cancelled() => {
                    // The token was cancelled

                }
                _ = dispatch_websocket_message(read, tx_receiver) => {
                }
            }
            println!("Exiting ha_msg_dispatcher_handler");
        });

        let ct_send_hndl = ct.clone();
        let ha_msg_sender_handler = tokio::spawn(async move {
            select! {
                _ = ct_send_hndl.cancelled() => {
                    // The token was cancelled

                }
                _ = send_websocket_message_handler(write, rx_sender) => {
                }
            }
            println!("Exiting ha_msg_sender_handler");
        });

        let client = HaConnection {
            // url: String::from(url),
            handle_message_handler: Some(handle_message_handler),
            ha_msg_dispatcher_handler: Some(ha_msg_dispatcher_handler),
            ha_msg_sender_handler: Some(ha_msg_sender_handler),
            tx_sender,
            ct,
            on_connected_cb: self.on_connected_cb.take(),
            current_id: 0,
            subscribers: Vec::new(),
        };

        println!("{}", "Successfully connected to Home Assistant!".green());
        return Ok(client);
    }
}

pub struct HaConnection {
    handle_message_handler: Option<JoinHandle<()>>,
    ha_msg_dispatcher_handler: Option<JoinHandle<()>>,
    ha_msg_sender_handler: Option<JoinHandle<()>>,
    tx_sender: Sender<String>,
    ct: CancellationToken,
    on_connected_cb: Option<Box<dyn FnMut()>>,
    current_id: u32,
    subscribers: Vec<Sender<String>>,
}

impl HaConnection {
    pub async fn wait_to_complete(&mut self) {
        if let Some(handle_message_handler) = self.handle_message_handler.take() {
            handle_message_handler.await.unwrap();
        }
        if let Some(ha_msg_dispatcher_handler) = self.ha_msg_dispatcher_handler.take() {
            ha_msg_dispatcher_handler.await.unwrap();
        }
        if let Some(ha_msg_sender_handler) = self.ha_msg_sender_handler.take() {
            ha_msg_sender_handler.await.unwrap();
        }
    }

    pub async fn subscribe_to_events(&mut self, event_type: &str) {
        self.current_id += 1;
        let subscribe_event = new_subscribe_to_events_command(self.current_id, event_type);
        self.tx_sender
            .send(subscribe_event.to_string())
            .await
            .unwrap();
        let event_stream = HaEventStream::new();
        self.subscribers.push(event_stream.sender_clone());
    }
}
impl Drop for HaConnection {
    fn drop(&mut self) {
        self.ct.cancel();
        // Small hack to wait for the tasks to complete
        // Todo: investigate how to do this properly
        sleep(std::time::Duration::from_millis(100));
    }
}

async fn authorize_home_assistant(
    token: &str,
    ws_reader: &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    ws_writer: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
) -> HassResult<()> {
    if let aut_req_msg = get_response_json(ws_reader).await? {
        if let Response::AuthRequired(aut_req_msg) = aut_req_msg {
            println!(
                "Connected to Home Assistant version {}",
                aut_req_msg.ha_version
            );
        } else {
            bail!("Unexpected response type");
        }
    }

    let auth_msg = json!(
    {
        "type": "auth",
        "access_token": token
    });
    let auth_msg = auth_msg.to_string();

    ws_writer.send(Message::Text(auth_msg)).await?;

    match get_response_json(ws_reader).await? {
        Response::AuthOk(auth_ok_msg) => {
            println!("Successfully authenticated to Home Assistant");
        }
        Response::AuthInvalid(auth_invalid_msg) => {
            bail!("Authentication failed: {}", auth_invalid_msg.message);
        }
        _ => {
            bail!("Unexpected response type");
        }
    }

    return Ok(());
}

async fn send_websocket_message_handler(
    mut write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    mut rx_sender: Receiver<String>,
) {
    while let Some(message) = rx_sender.recv().await {
        write.send(Message::Text(message)).await.unwrap();
    }
    drop(rx_sender);
    println!("Exiting send_websocket_message_handler");
}

// Dispatches messages from the websocket read stream and put them on a channel
async fn dispatch_websocket_message(
    ws_reader: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    tx_receiver: Sender<String>,
) {
    ws_reader
        .for_each(|message| async {
            let msg = message.unwrap();
            if msg.is_text() {
                let data = msg.into_text().unwrap();
                println!("got ws message: {}", data);
                tx_receiver.send(data).await.unwrap();
            } else {
                if msg.is_close() {
                    println!("Got close message");
                } else if msg.is_ping() {
                    println!("Got ping message")
                } else if msg.is_pong() {
                    println!("Got pong message")
                } else if msg.is_empty() {
                    println!("Got empty message");
                } else if msg.is_binary() {
                    println!("Got binary message");
                }
            }
        })
        .await;
    drop(tx_receiver);
    println!("Exiting dispatch_websocket_message")
}

// Reads the incoming messages from the channel and handles them accordingly
async fn handle_messages(mut ch_rx: mpsc::Receiver<String>, ch_tx: Sender<String>) {
    while let Some(data) = ch_rx.recv().await {
        println!("Got message: {}", data);
        // let event_json: Value = serde_json::from_str(&data).expect("JSON was not well-formatted");
        let ha_msg: Response = serde_json::from_str(&data).expect("JSON was not well-formatted");
        // let event_type = event_json["type"].as_str().unwrap();
        // match event_type {
        //     "event" => {
        //         let  event = &event_json["event"];
        //
        //         let  event: HaEvent =  serde_json::from_value(*event ).unwrap();
        //     }
        //     // "auth_required" => {
        //     //     println!("Authorizing with Home Assistant");
        //     //     authorize(&ch_tx, &config).await;
        //     // }
        //     &_ => { println!("unsupported event_type {:?}: Message: {:?}", event_type, event_json) }
        // }
    }
    drop(ch_tx);
    println!("Exiting handle_messages..")
}
pub(crate) async fn connect_ha_async(url: url::Url) -> HassResult<WebSocket> {
    let (client, _) = connect_async(url).await?;
    Ok(client)
}
