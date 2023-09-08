use std::time::Duration;

use r_hassclient::client::HaClient;
use r_hassclient::home_assistant::responses::WsEvent;
use r_hassclient::HaConnection;
use serde_json::json;
use tokio::signal;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    let addr = "ws://localhost:8124/api/websocket";
    let addr = url::Url::parse(addr).unwrap();
    let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiIwYmY3NjRmYWQ4MDM0ZjRmOWM2Y2E4ZDFhNTk0YWUzNCIsImlhdCI6MTY5Mjk3ODU1MiwiZXhwIjoyMDA4MzM4NTUyfQ.SvYtql9kB1MGZnEbBAtLX4EtrFktNUUCLMTAtQbg6FY";

    let mut client = HaClient::builder().build();
    let mut conn = client
        .connect_async(addr)
        .await
        .expect("Error connecting to Home Assistant!");

    if let Err(err) = conn.authenticate_with_token(token).await {
        println!("Failed to login to Home Assistant, {}", err);
        return;
    }
    let pet = |item: WsEvent| {
        println!("Closure is executed Event: {:?}", item);
    };
    if let Err(err) = conn.subscribe_message("state_changed", pet).await {
        println!("Error subscribing to event: {}", err);
        return;
    }

    do_stuff(conn).await;

    match signal::ctrl_c().await {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
        }
    }

    println!("Exit R-HassClient");
}

async fn do_stuff(mut conn: HaConnection) {
    tokio::spawn(async move {
        loop {
            if let Err(err) = conn
                .call_service(
                    "input_boolean".to_owned(),
                    "toggle".to_owned(),
                    Some(json!({"entity_id":"input_boolean.test"})),
                )
                .await
            {
                println!("Failed to call service: {:?}", err);
                break;
            }
            sleep(Duration::from_millis(1000)).await;
        }
    });
}
