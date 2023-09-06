use r_hassclient::client::HaClient;
use tokio::signal;

pub mod home_assistant;

#[tokio::main]
async fn main() {
    let addr = "ws://localhost:8124/api/websocket";
    let addr = url::Url::parse(addr).unwrap();
    let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiIwYmY3NjRmYWQ4MDM0ZjRmOWM2Y2E4ZDFhNTk0YWUzNCIsImlhdCI6MTY5Mjk3ODU1MiwiZXhwIjoyMDA4MzM4NTUyfQ.SvYtql9kB1MGZnEbBAtLX4EtrFktNUUCLMTAtQbg6FY";

    let mut client = HaClient::builder().build();
    match client.connect_async(addr).await {
        Err(err) => {
            println!("Failed to connect to Home Assistant, {}", err);
            return;
        }
        Ok(mut conn) => {
            if let Err(err) = conn.authenticate_with_token(token).await {
                println!("Failed to login to Home Assistant, {}", err);
                return;
            }
        }
    }
    match signal::ctrl_c().await {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
        }
    }
}

