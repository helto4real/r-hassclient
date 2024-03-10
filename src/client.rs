use colored::Colorize;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};

use crate::{
    Ask, Auth, CallService, HaCommand, HassConfig, HassError, HassResult, Response, Subscribe,
    WsEvent,
};

pub(crate) type WsSink = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
pub(crate) type WsStream = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
pub(crate) type HaListener = Arc<Mutex<HashMap<u64, Box<dyn Fn(WsEvent) + Send>>>>;
pub struct HaClient {}

#[derive(Default)]
pub struct HaClientBuilder {}

impl HaClientBuilder {
    pub fn new() -> HaClientBuilder {
        HaClientBuilder {}
    }
    pub fn build(self) -> HaClient {
        HaClient {}
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

        let (sink, stream): (WsSink, WsStream) = ha_ws.split();
        // Channel to send commands from client to Home Assistant
        let (to_ha, from_client) = mpsc::channel::<HaCommand>(20);

        // Channel to reveive events from Home Assistant to client
        let (to_client, from_ha) = mpsc::channel::<HassResult<Response>>(20);

        let event_listeners = Arc::new(Mutex::new(HashMap::new()));
        let event_listeners_clone_receiver = Arc::clone(&event_listeners);

        // Message id for HA messaging
        let last_msg_id = Arc::new(AtomicU64::new(1));
        let last_msg_clone_sender = Arc::clone(&last_msg_id);

        // Client --> Home Assistant
        if let Err(e) = sender_loop(last_msg_clone_sender, sink, from_client).await {
            to_client.send(Err(e)).await?
        }

        receiver_loop(stream, to_client, event_listeners_clone_receiver).await?;
        let last_sequence = Arc::new(AtomicU64::new(1));

        println!("{}", "Successfully connected to Home Assistant!".green());
        let ha_conn = HaConnection {
            to_ha,
            from_ha,
            event_listeners,
            last_sequence,
        };
        Ok(ha_conn)
    }
}

pub struct HaConnection {
    pub(crate) to_ha: Sender<HaCommand>,
    pub(crate) from_ha: Receiver<HassResult<Response>>,
    event_listeners: HaListener,
    // holds the id of the WS message
    last_sequence: Arc<AtomicU64>,
}

impl HaConnection {
    /// Authenticte with Home Assistant using the access token.
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

        let auth_cmd = HaCommand::AuthInfo(Auth {
            msg_type: "auth".to_owned(),
            access_token: token.to_owned(),
        });

        let response = self.send_command(auth_cmd).await?;

        //Check if the authetication was succefully, should receive {"type": "auth_ok"}
        match response {
            Response::AuthOk(_) => Ok(()),
            Response::AuthInvalid(err) => Err(HassError::AuthenticationFailed(err.message)),
            _ => Err(HassError::UnknownPayloadReceived),
        }
    }
    //used to subscribe to the event and if the subscribtion succeded the callback is registered
    pub async fn subscribe_message<F>(
        &mut self,
        event_name: &str,
        callback: F,
    ) -> HassResult<String>
    where
        F: Fn(WsEvent) + Send + 'static,
    {
        let id = get_last_seq(&self.last_sequence).expect("could not read the Atomic value");
        //create the Event Subscribe Command
        let cmd = HaCommand::SubscribeEvent(Subscribe {
            id: Some(id),
            msg_type: "subscribe_events".to_owned(),
            event_type: event_name.to_owned(),
        });

        //send command to subscribe to specific event
        let response = self.send_command(cmd).await.unwrap();

        //Add the callback in the event_listeners hashmap if the Subscription Response is successfull
        match response {
            Response::Result(v) if v.success => {
                let mut table = self.event_listeners.lock().await;
                table.insert(v.id, Box::new(callback));
                Ok("Ok".to_owned())
            }
            Response::Result(v) if !v.success => Err(HassError::ResponseError(v)),
            _ => Err(HassError::UnknownPayloadReceived),
        }
    }

    pub async fn ping(&mut self) -> HassResult<String> {
        let id = get_last_seq(&self.last_sequence).expect("could not read the Atomic value");

        //Send Ping command and expect Pong
        let ping_req = HaCommand::Ping(Ask {
            id: Some(id),
            msg_type: "ping".to_owned(),
        });

        let response = self.send_command(ping_req).await?;

        //Check the response, if the Pong was received
        match response {
            Response::Pong(_v) => Ok("pong".to_owned()),
            Response::Result(err) => Err(HassError::ResponseError(err)),
            _ => Err(HassError::UnknownPayloadReceived),
        }
    }

    /// This will get the current config of the Home Assistant.
    ///
    /// The server will respond with a result message containing the config.

    // pub async fn get_config(&mut self) -> HassResult<HassConfig> {
    //     let id = get_last_seq(&self.last_sequence).expect("could not read the Atomic value");
    //
    //     //Send GetConfig command and expect Pong
    //     let config_req = HaCommand::GetConfig(Ask {
    //         id: Some(id),
    //         msg_type: "get_config".to_owned(),
    //     });
    //     let response = self.command(config_req).await?;
    //
    //     match response {
    //         Response::Result(data) => match data.success {
    //             true => {
    //                 let config: HassConfig = serde_json::from_value(
    //                     data.result.expect("Expecting to get the HassConfig"),
    //                 )?;
    //                 return Ok(config);
    //             }
    //             false => return Err(HassError::ReponseError(data)),
    //         },
    //         _ => return Err(HassError::UnknownPayloadReceived),
    //     }
    // }

    pub async fn call_service(
        &mut self,
        domain: String,
        service: String,
        service_data: Option<Value>,
    ) -> HassResult<String> {
        let id = get_last_seq(&self.last_sequence).expect("could not read the Atomic value");
        //Send GetStates command and expect a number of Entities
        let services_req = HaCommand::CallService(CallService {
            id: Some(id),
            msg_type: "call_service".to_owned(),
            domain,
            service,
            service_data,
        });
        let response = self.send_command(services_req).await?;

        match response {
            Response::Result(data) => match data.success {
                true => Ok("command executed successfully".to_owned()),
                false => Err(HassError::ResponseError(data)),
            },
            _ => Err(HassError::UnknownPayloadReceived),
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

        // Receive response from Home Assistant
        self.from_ha
            .recv()
            .await
            .ok_or_else(|| HassError::ConnectionError)?
    }
}

fn get_last_seq(last_sequence: &Arc<AtomicU64>) -> Option<u64> {
    // Increase the last sequence and use the previous value in the request
    match last_sequence.fetch_add(1, Ordering::Relaxed) {
        0 => None,
        v => Some(v),
    }
}

//listen for client commands, convert it to Message and send it to HA through websocket
async fn sender_loop(
    last_sequence: Arc<AtomicU64>,
    mut sink: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    mut from_client: Receiver<HaCommand>,
) -> HassResult<()> {
    tokio::spawn(async move {
        loop {
            if let Some(item) = from_client.recv().await {
                match item {
                    HaCommand::AuthInfo(auth) => {
                        // Transform command to Message
                        let cmd = HaCommand::AuthInfo(auth).to_tungstenite_message();

                        // Send the message to HA
                        if let Err(e) = sink.send(cmd).await {
                            return HassError::TungsteniteError(e);
                        }
                    }

                    HaCommand::Ping(mut ping) => {
                        ping.id = get_last_seq(&last_sequence);

                        // Transform command to Message
                        let cmd = HaCommand::Ping(ping).to_tungstenite_message();

                        // Send the message to gateway
                        if let Err(e) = sink.send(cmd).await {
                            return HassError::TungsteniteError(e);
                        }
                    }

                    HaCommand::SubscribeEvent(mut subscribe) => {
                        subscribe.id = get_last_seq(&last_sequence);

                        // Transform command to Message
                        let cmd = HaCommand::SubscribeEvent(subscribe).to_tungstenite_message();

                        // Send the message to gateway
                        if let Err(e) = sink.send(cmd).await {
                            return HassError::TungsteniteError(e);
                        }
                    } // Command::Unsubscribe(mut unsubscribe) => {
                    //     unsubscribe.id = get_last_seq(&last_sequence);
                    //
                    //     // Transform command to Message
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
                    //     // Transform command to Message
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
                    //     // Transform command to Message
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
                    //     // Transform command to Message
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
                    //     // Transform command to Message
                    //     let cmd = Command::GetServices(getpanels).to_tungstenite_message();
                    //
                    //     // Send the message to gateway
                    //     if let Err(e) = sink.send(cmd).await {
                    //         return Err(HassError::from(e));
                    //     }
                    // }
                    HaCommand::CallService(mut callservice) => {
                        callservice.id = get_last_seq(&last_sequence);

                        // Transform command to Message
                        let cmd = HaCommand::CallService(callservice).to_tungstenite_message();

                        // Send the message to gateway
                        if let Err(e) = sink.send(cmd).await {
                            return HassError::TungsteniteError(e);
                        }
                    }
                }
            } else {
                return HassError::GenericError("client channel is closed".to_string());
            }
        }
    });
    Ok(())
}

//listen for Home Assistant responses and either send to client the response or execute the defined closure for Event subscribtion
async fn receiver_loop(
    //    last_sequence: Arc<AtomicU64>,
    mut stream: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    to_client: Sender<HassResult<Response>>,
    event_listeners: HaListener,
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
                                    let mut table = event_listeners.lock().await;

                                    match table.get_mut(&event.id) {
                                        Some(client_func) => {
                                            //execute client closure
                                            client_func(event);
                                        }
                                        None => todo!("send unsubscribe request"),
                                    }
                                }
                                _ => {
                                    to_client.send(Ok(value)).await.unwrap();
                                }
                            },
                            Err(error) => to_client.send(Err(error)).await.unwrap(),
                        };
                    }
                    // Just ignore these messages for now, I keep all variants for clearer code
                    // what is ignored
                    Message::Binary(_) => { /*ignore*/ }
                    Message::Ping(_) => { /*ignore*/ }
                    Message::Pong(_) => { /*ignore*/ }
                    Message::Close(_) => { /*ignore*/ }
                    Message::Frame(_) => { /*ignore*/ }
                },

                Some(Err(error)) => {
                    eprintln!("Error!!: {:?}", error);
                    match to_client
                        .send(Err(HassError::TungsteniteError(error)))
                        .await
                    {
                        //send the error to client ("unexpected message format, like a new error")
                        Ok(_r) => {}
                        Err(_e) => {}
                    }
                }
                None => {}
            }
        }
    });
    Ok(())
}
