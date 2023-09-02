use r_hassclient::client::HaClient;
use tokio::signal;

pub mod home_assistant;

#[tokio::main]
async fn main() {
    // let  jsn = json!({"id":1,"type":"event","event":{"event_type":"state_changed","data":{"entity_id":"input_boolean.test","old_state":{"entity_id":"input_boolean.test","state":"off","attributes":{"editable":true,"friendly_name":"test"},"last_changed":"2023-08-28T09:08:13.985677+00:00","last_updated":"2023-08-28T09:08:13.985677+00:00","context":{"id":"01H8XPD611JJ7Q3WP5VT5FVEWN","parent_id":null,"user_id":"f89f13024806490b8d879160843ddf54"}},"new_state":{"entity_id":"input_boolean.test","state":"on","attributes":{"editable":true,"friendly_name":"test"},"last_changed":"2023-08-28T09:09:05.471838+00:00","last_updated":"2023-08-28T09:09:05.471838+00:00","context":{"id":"01H8XPER9ZAGWM4P3WZQ7BPKPR","parent_id":null,"user_id":"f89f13024806490b8d879160843ddf54"}}},"origin":"LOCAL","time_fired":"2023-08-28T09:09:05.471838+00:00","context":{"id":"01H8XPER9ZAGWM4P3WZQ7BPKPR","parent_id":null,"user_id":"f89f13024806490b8d879160843ddf54"}}});
    // // let  result_jsn = json!({"id":1,"type":"result","success":true,"result":null});
    // // let  json_unknown = json!({"id":1,"type":"unknown"});
    // let  ha_msg: Response = serde_json::from_value(jsn).unwrap();
    //
    // println!("{:?}", ha_msg);
    //
    let addr = "ws://localhost:8124/api/websocket";
    let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiIwYmY3NjRmYWQ4MDM0ZjRmOWM2Y2E4ZDFhNTk0YWUzNCIsImlhdCI6MTY5Mjk3ODU1MiwiZXhwIjoyMDA4MzM4NTUyfQ.SvYtql9kB1MGZnEbBAtLX4EtrFktNUUCLMTAtQbg6FY";

    let mut client = HaClient::builder().build();

    let mut ha_connection = match client.connect_ha_async(&addr, &token).await {
        Ok(c) => c,
        Err(e) => {
            panic!("Failed to connect, error {}", e)
        }
    };

    ha_connection.subscribe_to_events("state_changed").await;

    match signal::ctrl_c().await {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
            // we also shut down in case of error
        }
    }
}
