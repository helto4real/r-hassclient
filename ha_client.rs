use futures_util::{SinkExt, StreamExt};
use futures_util::stream::{SplitSink, SplitStream};
use serde_json::{Result, Value};
use serde_json::json;
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;
use tokio_tungstenite::{connect_async, MaybeTlsStream, tungstenite::protocol::Message, WebSocketStream};
use tokio_util::sync::CancellationToken;

pub struct HaClient {
    // url: String,
    handle_message_handler: Option<JoinHandle<()>>,
    ha_msg_dispatcher_handler: Option<JoinHandle<()>>,
    ha_msg_sender_handler: Option<JoinHandle<()>>,
    tx_sender: Sender<String>,
    ct: CancellationToken,
    on_connected_cb: Option<Box<dyn FnMut()>>,
}

impl Drop for HaClient {
    fn drop(&mut self) {
        self.ct.cancel();
    }
}

pub struct ReceiverConfig {
    token: String,
}

impl ReceiverConfig {
    pub fn new(token: String) -> Self {
        Self { token }
    }
}


pub async fn connect_ha_async(url: &str, token: &str) -> Result<HaClient> {
    let uri = url::Url::parse(&url).expect("Failed to parse url");
    let (ws_stream, _) = connect_async(uri).await.expect("Failed to connect to websocket!");
    let (write, read): (SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>, SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>) = ws_stream.split();

    // channel to handle all new messages from the websocket
    let (tx_receiver, rx_receiver): (Sender<String>, Receiver<String>) = mpsc::channel(100);
    // channel to handle all messages that should be sent on the websocket
    let (tx_sender, rx_sender): (Sender<String>, Receiver<String>) = mpsc::channel(100);
    let recv_config = ReceiverConfig::new(String::from(token));

    let cloned_sender = tx_sender.clone();
    let ct = CancellationToken::new();
    let ct_msg_hndl = ct.clone();
    let handle_message_handler = tokio::spawn(async move {
        select! {
            _ = ct_msg_hndl.cancelled() => {
                // The token was cancelled
                
            }
            _ = handle_messages(rx_receiver, cloned_sender, recv_config) => {
            }
        }
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
    });

    let client = HaClient {
        // url: String::from(url),
        handle_message_handler: Some(handle_message_handler),
        ha_msg_dispatcher_handler: Some(ha_msg_dispatcher_handler),
        ha_msg_sender_handler: Some(ha_msg_sender_handler),
        tx_sender,
        ct,
        on_connected_cb: None
    };
    return Ok(client);
}

async fn send_websocket_message_handler(mut write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>, mut rx_sender: Receiver<String>) {
    while let Some(message) = rx_sender.recv().await {
        write.send(Message::Text(message)).await.unwrap();
    }
    drop(rx_sender);
    println!("Exiting send_websocket_message_handler");
}

// Dispatches messages from the websocket read stream and put them on a channel
async fn dispatch_websocket_message(ws_reader: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>, tx_receiver: Sender<String>) {
    ws_reader.for_each(|message| async {
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
    }).await;
    drop(tx_receiver);
    println!("Exiting dispatch_websocket_message")
}

// Reads the incoming messages from the channel and handles them accordingly
async fn handle_messages(mut ch_rx: mpsc::Receiver<String>, ch_tx: Sender<String>, config: ReceiverConfig) {
    while let Some(data) = ch_rx.recv().await {
        println!("Got message: {}", data);
        let event_json: Value = serde_json::from_str(&data).expect("JSON was not well-formatted");
        let event_type = event_json["type"].as_str().unwrap();
        match event_type {
            "auth_required" => {
                println!("Authorizing with Home Assistant");
                authorize(&ch_tx, &config).await;
            }
            &_ => { println!("unsupported event_type {:?}: Message: {:?}", event_type, event_json) }
        }
    }
    drop(ch_tx);
    println!("Exiting handle_messages..")
}

async fn authorize(ch_tx: &Sender<String>, config: &ReceiverConfig) {
    let auth_msg = json!(
    {
        "type": "auth",
        "access_token": config.token
    });
    let auth_msg = auth_msg.to_string();
    println!("Sending authmsg: {}", auth_msg);
    if ch_tx.is_closed() {
        println!("Send channel is closed");
        return;
    }
    match ch_tx.send(auth_msg).await {
        Err(err) => { eprintln!("Failed to send message to channel {}", err) }
        Ok(()) => {}
    }
}

impl HaClient {
    pub async fn run(&mut self) {
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
}
