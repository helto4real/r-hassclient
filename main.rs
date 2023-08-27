
pub mod ha_client;

#[tokio::main]
async fn main() {
    let addr = "ws://localhost:8124/api/websocket";
    let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiIwYmY3NjRmYWQ4MDM0ZjRmOWM2Y2E4ZDFhNTk0YWUzNCIsImlhdCI6MTY5Mjk3ODU1MiwiZXhwIjoyMDA4MzM4NTUyfQ.SvYtql9kB1MGZnEbBAtLX4EtrFktNUUCLMTAtQbg6FY";

    let mut ha_client = ha_client::connect_ha_async(addr, token).await.expect("Failed to connect!");
    ha_client.run().await;
}
