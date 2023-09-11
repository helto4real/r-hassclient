use std::{collections::HashMap, time::Duration};
use std::{future::Future, thread};
use r_hassclient::HaClient;
use serde_json::json;
use testcontainers::{core::WaitFor, *};
use tokio::{time::sleep, sync::{Mutex, mpsc::{UnboundedSender, UnboundedReceiver, self}}, runtime};
use ctor::{ctor, dtor};

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
}

#[derive(Debug)]
enum ContainerCommands {
    FetchHaConnectionData,
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
    HA_CONTAINER_COMMANDS.tx.send(ContainerCommands::Stop).unwrap();
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

    println!("Sending post on url: {} with json {}", url, json);

    let client = reqwest::Client::new();

    let resp = client.post(url).json(&json).send().await.unwrap();

    assert_eq!(resp.status(), 200);
    let resp = resp.json::<serde_json::Value>().await.unwrap();

    println!("RAW: {:?}", resp);
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
            ContainerCommands::FetchHaConnectionData => HA_CONNECTION_INFO.tx.send((port, access_token.to_string())).unwrap(),
            ContainerCommands::Stop => {
                container.stop();
                HA_STOP_CONTAINER.tx.send(()).unwrap();
                rx.close();
            }
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn should_be_able_to_login_with_access_token() {
    // Send command to get the current test container info
    HA_CONTAINER_COMMANDS.tx.send(ContainerCommands::FetchHaConnectionData).unwrap();
    let (port, access_token) = HA_CONNECTION_INFO.rx.lock().await.recv().await.unwrap();
    
    let addr = format!("ws://localhost:{port}/api/websocket");
    let addr = url::Url::parse(&addr).unwrap();

    let mut client = HaClient::builder().build();
    let mut conn = client
        .connect_async(addr)
        .await
        .expect("Error connecting to Home Assistant!");

    conn.authenticate_with_token(&access_token).await.expect("Failed to authenticate with Home Assistant");
}
