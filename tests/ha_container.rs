use ctor::{ctor, dtor};
use r_hassclient::client::HaConnection;
use r_hassclient::{HaClient, HaEventData, HassResult, WsEvent};
use serde_json::json;
use std::borrow::Borrow;
use std::fmt::format;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::{collections::HashMap, time::Duration};
use std::{future::Future, thread};
use testcontainers::{core::WaitFor, *};
use tokio::sync::oneshot;
use tokio::time::timeout;
use tokio::{
    runtime,
    sync::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        Mutex,
    },
    time::sleep,
};

#[macro_use]
extern crate lazy_static;

// The test container Home Assistant image definition
pub struct HaImage;
impl Image for HaImage {
    type Args = ();

    fn name(&self) -> String {
        "homeassistant/home-assistant".to_owned()
    }

    fn tag(&self) -> String {
        "stable".to_owned()
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stderr(
            "service legacy-services successfully started",
        )]
    }

    fn expose_ports(&self) -> Vec<u16> {
        vec![8123]
    }
}

lazy_static! {
    static ref HA_CONTAINER_COMMANDS: Channel<ContainerCommands> = channel();
    static ref HA_CONNECTION_INFO: Channel<(u16, String)> = channel();
    static ref HA_STOP_CONTAINER: Channel<()> = channel();
    static ref HA_SETUP_TEST_DATA: Channel<()> = channel();
}

#[derive(Debug)]
enum ContainerCommands {
    FetchHaConnectionData,
    // AddTestData,
    Stop,
}

struct Channel<T> {
    tx: UnboundedSender<T>,
    rx: Mutex<UnboundedReceiver<T>>,
}

fn channel<T>() -> Channel<T> {
    let (tx, rx) = mpsc::unbounded_channel();
    Channel {
        tx,
        rx: Mutex::new(rx),
    }
}

// Called before all tests are run. Spawns the function that starts the container
#[ctor]
fn on_startup() {
    thread::spawn(|| execute_blocking(start_container()));
}

// Calls when all tests are run and make sure the container is stopped
#[dtor]
fn on_shutdown() {
    execute_blocking(clean_up());
}

// Clean up container by sending a command and wait for it to finish
async fn clean_up() {
    HA_CONTAINER_COMMANDS
        .tx
        .send(ContainerCommands::Stop)
        .unwrap();
    HA_STOP_CONTAINER.rx.lock().await.recv().await;
    println!("HA container stopped.")
}

// Start the tokio runtime with default features used
fn execute_blocking<F: Future>(f: F) {
    runtime::Builder::new_current_thread()
        .enable_time()
        .enable_io()
        .build()
        .unwrap()
        .block_on(f);
}

/// Starts the container and waits for commands from tests to get connection information
///
/// The way we can handle one instance of testcontainer is to spawn a thread that starts
/// the container. Then waits for the tests go ask for connection information. The tests
/// will block until the container is ready.
///
/// Home Assistant is provitioned with default user and a access_token to use in all tests.
///
/// # Panics
///
/// Panics if start the container fails
async fn start_container() {
    let docker = clients::Cli::default();

    let container = docker.run(HaImage);
    println!("Test container running");

    let port = container.get_host_port_ipv4(8123);

    println!("Connected on port: {}", port);

    sleep(Duration::from_millis(5000)).await;
    let json = json!(
    {
        "client_id": "http://dummyClientId",
        "name": "foobar",
        "username": "test-user",
        "password": "P@ssword!",
        "language": "en-GB"
    });

    let url = format!("http://localhost:{}/api/onboarding/users", port);

    // println!("Sending post on url: {} with json {}", url, json);

    let client = reqwest::Client::new();

    let resp = client.post(url).json(&json).send().await.unwrap();

    assert_eq!(resp.status(), 200);
    let resp = resp.json::<serde_json::Value>().await.unwrap();

    // println!("RAW: {:?}", resp);
    let auth_code = resp.get("auth_code").unwrap().as_str().unwrap();

    println!(
        "Created user: test-user, password = 'P@ssword' Got auth_code = {}",
        auth_code
    );

    let mut auth_params = HashMap::new();
    auth_params.insert("client_id", "http://dummyClientId");
    auth_params.insert("code", auth_code);
    auth_params.insert("grant_type", "authorization_code");

    let auth_url = format!("http://localhost:{}/auth/token", port);

    let resp = client
        .post(auth_url)
        .form(&auth_params)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    let resp = resp.json::<serde_json::Value>().await.unwrap();

    let access_token = resp.get("access_token").unwrap().as_str().unwrap();

    println!("ACCESS TOKEN: {}", access_token);
    let mut rx = HA_CONTAINER_COMMANDS.rx.lock().await;
    while let Some(command) = rx.recv().await {
        println!("Received container command: {:?}", command);
        match command {
            ContainerCommands::FetchHaConnectionData => HA_CONNECTION_INFO
                .tx
                .send((port, access_token.to_string()))
                .unwrap(),
            // ContainerCommands::AddTestData => {
            //     if let Err(e) = setup_test_data(&client, port, access_token).await {
            //         panic!("Failed to setup test data: {}", e);
            //     }
            //     HA_SETUP_TEST_DATA.tx.send(()).unwrap();
            // }
            ContainerCommands::Stop => {
                container.stop();
                HA_STOP_CONTAINER.tx.send(()).unwrap();
                rx.close();
            }
        }
    }
}

// async fn setup_test_data(client: &reqwest::Client, port: u16,  access_token: &str) -> Result<(), reqwest::Error> {
//     let url = format!("http://localhost:{}/api/states/intput_boolean.test", port);
//     let json = json!({
//         "state": "off"
//     });
//     println!("Setting up test data: {}, token: {}", url, access_token);
//     let resp = client
//         .post(url)
//         .header("Authorization", format!("Bearer {}", access_token))
//         .json(&json)
//         .send()
//         .await
//         .unwrap();
//
//     assert_eq!(resp.status(), 201);
//
//     Ok(())
// }

#[tokio::test(flavor = "multi_thread")]
async fn should_be_able_to_send_ping_message() {
    let mut conn = match connect_to_home_assistant().await {
        Err(err) => {
            panic!("Failed to connect to Home Assistant: {}", err);
        }
        Ok(conn) => conn,
    };
    conn.ping().await.expect("Failed to send ping message");
}

async fn connect_to_home_assistant() -> HassResult<HaConnection> {
    // Send command to get the current test container info
    HA_CONTAINER_COMMANDS
        .tx
        .send(ContainerCommands::FetchHaConnectionData)
        .unwrap();
    let (port, access_token) = HA_CONNECTION_INFO.rx.lock().await.recv().await.unwrap();

    let addr = format!("ws://localhost:{port}/api/websocket");
    let addr = url::Url::parse(&addr).unwrap();

    let mut client = HaClient::builder().build();
    let mut conn = client
        .connect_async(addr)
        .await
        .expect("Error connecting to Home Assistant!");

    conn.authenticate_with_token(&access_token)
        .await
        .expect("Failed to authenticate with Home Assistant");

    Ok(conn)
}

#[tokio::test(flavor = "multi_thread")]
async fn should_be_able_to_subscribe_to_events() {
    let mut conn = match connect_to_home_assistant().await {
        Err(err) => {
            panic!("Failed to connect to Home Assistant: {}", err);
        }
        Ok(conn) => conn,
    };

    // Create a helper to test with
    if let Err(helper_res) = conn.create_helper("input_boolean", "test").await {
        panic!("Failed to create input_boolean helper: {}", helper_res);
    }

    let (tx, mut rx) = mpsc::channel(2);

    let pet = move |item: WsEvent| {
        println!("EVENT:");
        let tx_clone = tx.clone();
        let event_data = item.event.get_event_data();
        match event_data {
            Ok(HaEventData::StateChangedEvent(event)) => {
                println!("State changed event:");
                tokio::spawn(async move {
                    let _ = tx_clone.send(()).await;
                });
                println!("{}", event);
                // panic!("Should not receive state changed event");
            }
            Err(err) => {
                println!("Error parsing event data: {}", err);
            }
        }
    };

    if let Err(err) = conn.subscribe_message("state_changed", pet).await {
        println!("Failed to subscribe to state_changed events: {}", err);
        return;
    }

    let res = conn
        .call_service(
            "input_boolean".to_owned(),
            "toggle".to_owned(),
            Some(json!({"entity_id":"input_boolean.test"})),
        )
        .await;

    if let Err(err) = res {
        println!("Failed to call service: {}", err);
    }

    // Set a timeout for waiting on the callback.
    tokio::select! {
        _ = tokio::time::sleep(Duration::from_millis(2000)) => {
            panic!("Timeout waiting for state_changed event");
        }
       _= rx.recv() => { },
    }
}
