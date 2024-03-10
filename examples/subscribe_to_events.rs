use std::env::var;

// use std::time::Duration;
use lazy_static::lazy_static;
use r_hassclient::{client::HaClient, HaEventData, WsEvent};
// use r_hassclient::HaConnection;
// use serde_json::json;
use tokio::signal;
// use tokio::time::sleep;
lazy_static! {
    static ref TOKEN: String =
        var("HASS_TOKEN").expect("please set up the HASS_TOKEN env variable before running this");
}
#[tokio::main]
async fn main() {
    let addr = "ws://localhost:8124/api/websocket";
    let addr = url::Url::parse(addr).unwrap();

    let mut client = HaClient::builder().build();
    let mut conn = client
        .connect_async(addr)
        .await
        .expect("Error connecting to Home Assistant!");

    if let Err(err) = conn.authenticate_with_token(&TOKEN).await {
        println!("Failed to login to Home Assistant, {}", err);
        return;
    }

    let pet = |item: WsEvent| {
        let event_data = item.event.get_event_data();
        match event_data {
            Ok(HaEventData::StateChangedEvent(event)) => {
                println!("State changed event:");
                println!("{}", event);
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

    // do_stuff(conn).await;

    match signal::ctrl_c().await {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
        }
    }

    println!("Exit R-HassClient")
}

// async fn do_stuff(mut conn: HaConnection) {
//     tokio::spawn(async move {
//         loop {
//             if let Err(err) = conn
//                 .call_service(
//                     "input_boolean".to_owned(),
//                     "toggle".to_owned(),
//                     Some(json!({"entity_id":"input_boolean.test"})),
//                 )
//                 .await
//             {
//                 println!("Failed to call service: {:?}", err);
//                 break;
//             }
//             sleep(Duration::from_millis(1000)).await;
//         }
//     });
// }
