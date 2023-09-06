mod event_stream;

use crate::{
    home_assistant::{commands::*, responses::Response},
    types::{HassError, HassResult, WebSocket},
};
use colored::Colorize;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use std::sync::{atomic::AtomicU64, Arc};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};

pub struct HaClient {
    // url: String,
    //on_connected_cb: Option<Box<dyn FnMut()>>,
}

#[derive(Default)]
pub struct HaClientBuilder {
    //on_connected_cb: Option<Box<dyn FnMut()>>,
}

impl HaClientBuilder {
    pub fn new() -> HaClientBuilder {
        // Set the minimally required fields of Foo.
        HaClientBuilder {
          //  on_connected_cb: None,
        }
    }
    // pub fn on_connected(mut self, func: Box<dyn FnMut()>) -> HaClientBuilder {
    //     //self.on_connected_cb = Some(func);
    //     self
    // }
    pub fn build(self) -> HaClient {
        HaClient {
            //on_connected_cb: self.on_connected_cb,
        }
    }
}

impl HaClient {
    pub fn builder() -> HaClientBuilder {
        HaClientBuilder::default()
    }

    /// Connects to Home Assistant
    ///
    /// # Errors
    ///
    /// This function will return an error if the connection to Home Assistant fails
    pub async fn connect_async(&mut self, url: url::Url) -> HassResult<HaConnection> {
        let (ha_ws, _) = connect_async(url).await?;

        let (sink, stream): (
            SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
            SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
        ) = ha_ws.split();
        // Channel to send commands from client to Home Assistant
        let (to_ha, from_client) = mpsc::channel::<HaCommand>(20);

        // Channel to reveive events from Home Assistant to client
        let (mut to_client, from_ha) = mpsc::channel::<HassResult<Response>>(20);

        // Message id for HA messaging
        let last_msg_id = Arc::new(AtomicU64::new(1));
        let last_msg_clone_sender = Arc::clone(&last_msg_id);
        // Client --> Gateway
        if let Err(e) = sender_loop(last_msg_clone_sender, sink, from_client).await {
            to_client.send(Err(HassError::from(e))).await.unwrap();
        }
        if let Err(e) = receiver_loop(stream, to_client).await {
            return Err(e);
        };

        println!("{}", "Successfully connected to Home Assistant!".green());
        let ha_conn = HaConnection {
            to_ha,
            from_ha,
        };
        Ok(ha_conn)
    }
}

pub struct HaConnection {
    pub(crate) to_ha: Sender<HaCommand>,
    pub(crate) from_ha: Receiver<HassResult<Response>>,
}

impl HaConnection {
    /// Authenticte with Home Assistant using a token. 
    ///
    /// # Errors
    /// 
    /// This function will return an error if the autentication fails.
    pub async fn authenticate_with_token(&mut self, token: &str) -> HassResult<()> {

        _ = self
            .from_ha
            .recv()
            .await
            .ok_or_else(|| HassError::ConnectionError)?;

        let auth_cmd = HaCommand::Auth(Auth {
            msg_type: "auth".to_owned(),
            access_token: token.to_owned(),
        });
        
        let response = self.send_command(auth_cmd).await?;

        //Check if the authetication was succefully, should receive {"type": "auth_ok"}
        match response {
            Response::AuthOk(_) => Ok(()),
            Response::AuthInvalid(err) => return Err(HassError::AuthenticationFailed(err.message)),
            _ => return Err(HassError::UnknownPayloadReceived),
        }
    }
    /// Sends an command and waits for result.
    ///
    /// Since events are managed directly in callbacks the returning message must be related to the
    /// command. We do not need to check the id sent and id in the result from Home Assistant.
    ///
    /// # Errors
    ///
    /// This function will return an error if the channel is dropped.
    pub(crate) async fn send_command(&mut self, cmd: HaCommand) -> HassResult<Response> {
        // Send the command to Home Assistant
        self.to_ha
            .send(cmd)
            .await
            .map_err(|_| HassError::ConnectionError)?;

        // Receive response from command (todo: check id matches)
        self.from_ha
            .recv()
            .await
            .ok_or_else(|| HassError::ConnectionError)?
    }
}

//listen for client commands and transform those to TungsteniteMessage and send to gateway
async fn sender_loop(
    last_sequence: Arc<AtomicU64>,
    mut sink: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    mut from_client: Receiver<HaCommand>,
) -> HassResult<()> {
    tokio::spawn(async move {
        //Fuse the stream such that poll_next will never again be called once it has finished.
        //let mut fused = from_client.fuse();
        loop {
            match from_client.recv().await {
                Some(item) => match item {
                    // Command::Close => {
                    //     return sink
                    //         .send(TungsteniteMessage::Close(None))
                    //         .await
                    //         .map_err(|_| HassError::ConnectionClosed);
                    // }
                    HaCommand::Auth(auth) => {
                        // Transform command to TungsteniteMessage
                        let cmd = HaCommand::Auth(auth).to_tungstenite_message();

                        // Send the message to gateway
                        if let Err(e) = sink.send(cmd).await {
                            return HassError::TungsteniteError(e);
                        }
                    } // Command::Ping(mut ping) => {
                      //     ping.id = get_last_seq(&last_sequence);
                      //
                      //     // Transform command to TungsteniteMessage
                      //     let cmd = Command::Ping(ping).to_tungstenite_message();
                      //
                      //     // Send the message to gateway
                      //     if let Err(e) = sink.send(cmd).await {
                      //         return Err(HassError::from(e));
                      //     }
                      // }
                      // Command::SubscribeEvent(mut subscribe) => {
                      //     subscribe.id = get_last_seq(&last_sequence);
                      //
                      //     // Transform command to TungsteniteMessage
                      //     let cmd = Command::SubscribeEvent(subscribe).to_tungstenite_message();
                      //
                      //     // Send the message to gateway
                      //     if let Err(e) = sink.send(cmd).await {
                      //         return Err(HassError::from(e));
                      //     }
                      // }
                      // Command::Unsubscribe(mut unsubscribe) => {
                      //     unsubscribe.id = get_last_seq(&last_sequence);
                      //
                      //     // Transform command to TungsteniteMessage
                      //     let cmd = Command::Unsubscribe(unsubscribe).to_tungstenite_message();
                      //
                      //     // Send the message to gateway
                      //     if let Err(e) = sink.send(cmd).await {
                      //         return Err(HassError::from(e));
                      //     }
                      // }
                      // Command::GetConfig(mut getconfig) => {
                      //     getconfig.id = get_last_seq(&last_sequence);
                      //
                      //     // Transform command to TungsteniteMessage
                      //     let cmd = Command::GetConfig(getconfig).to_tungstenite_message();
                      //
                      //     // Send the message to gateway
                      //     if let Err(e) = sink.send(cmd).await {
                      //         return Err(HassError::from(e));
                      //     }
                      // }
                      // Command::GetStates(mut getstates) => {
                      //     getstates.id = get_last_seq(&last_sequence);
                      //
                      //     // Transform command to TungsteniteMessage
                      //     let cmd = Command::GetStates(getstates).to_tungstenite_message();
                      //
                      //     // Send the message to gateway
                      //     if let Err(e) = sink.send(cmd).await {
                      //         return Err(HassError::from(e));
                      //     }
                      // }
                      // Command::GetServices(mut getservices) => {
                      //     getservices.id = get_last_seq(&last_sequence);
                      //
                      //     // Transform command to TungsteniteMessage
                      //     let cmd = Command::GetServices(getservices).to_tungstenite_message();
                      //
                      //     // Send the message to gateway
                      //     if let Err(e) = sink.send(cmd).await {
                      //         return Err(HassError::from(e));
                      //     }
                      // }
                      // Command::GetPanels(mut getpanels) => {
                      //     getpanels.id = get_last_seq(&last_sequence);
                      //
                      //     // Transform command to TungsteniteMessage
                      //     let cmd = Command::GetServices(getpanels).to_tungstenite_message();
                      //
                      //     // Send the message to gateway
                      //     if let Err(e) = sink.send(cmd).await {
                      //         return Err(HassError::from(e));
                      //     }
                      // }
                      // Command::CallService(mut callservice) => {
                      //     callservice.id = get_last_seq(&last_sequence);
                      //
                      //     // Transform command to TungsteniteMessage
                      //     let cmd = Command::CallService(callservice).to_tungstenite_message();
                      //
                      //     // Send the message to gateway
                      //     if let Err(e) = sink.send(cmd).await {
                      //         return Err(HassError::from(e));
                      //     }
                      // }
                },
                None => {}
            }
        }
    });

    Ok(())
}

//listen for gateway responses and either send to client the response or execute the defined closure for Event subscribtion
async fn receiver_loop(
    //    last_sequence: Arc<AtomicU64>,
    mut stream: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    mut to_client: Sender<HassResult<Response>>,
    //event_listeners: Arc<Mutex<HashMap<u64, Box<dyn Fn(WSEvent) + Send>>>>,
) -> HassResult<()> {
    tokio::spawn(async move {
        loop {
            match stream.next().await {
                Some(Ok(item)) => match item {
                    Message::Text(data) => {
                        let payload: Result<Response, HassError> = serde_json::from_str(&data)
                            .map_err(|_| HassError::UnknownPayloadReceived);

                        //Match on payload, and act accordingly, like execute the client defined closure if any Event received
                        match payload {
                            Ok(value) => match value {
                                Response::Event(event) => {
                                    println!("Event: {:?}", event);
                                    // let mut table = event_listeners.lock().await;
                                    //
                                    // match table.get_mut(&event.id) {
                                    //     Some(client_func) => {
                                    //         //execute client closure
                                    //         client_func(event);
                                    //     }
                                    //     None => todo!("send unsubscribe request"),
                                    // }
                                }
                                _ => {
                                    println!("Received message: {:?}", value);
                                    to_client.send(Ok(value)).await.unwrap();
                                }
                            },
                            Err(error) => to_client.send(Err(error)).await.unwrap(),
                        };
                    }
                    _ => {}
                },

                Some(Err(error)) => match to_client
                    .send(Err(HassError::TungsteniteError(error)))
                    .await
                {
                    //send the error to client ("unexpected message format, like a new error")
                    Ok(_r) => {}
                    Err(_e) => {}
                },
                None => {}
            }
        }
    });
    Ok(())
}
