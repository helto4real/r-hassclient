use lazy_static::lazy_static;

use r_hassclient::{client::{HaClient, HaConnection}, HaEventData, WsEvent};

pub struct AppRuntime<'a> {
    pub is_connected: bool,
    pub conn: &'a Option<HaConnection>,
}



impl<'a> AppRuntime<'a> {
    pub(crate) async fn start(host: &str, port: u16, ssl: bool) -> Self {
        // setup the correct url to Home Assistant wegsocket API
        let protocol = if ssl { "wss" } else { "ws" };
        let addr = format!("{}://{}:{}/api/websocket", protocol, host, port); 
        let url = url::Url::parse(addr.as_str()).unwrap();

        let mut client = HaClient::builder().build();

        let runtime = AppRuntime {
            is_connected: false,
            conn: &None,
        };

        let mut conn = client
        .connect_async(url)
        .await;
        
        
        if let Err(err) = conn {
            println!("Failed to connect to Home Assistant, {}", err);
            return runtime
        }

    if let Err(err) = conn.authenticate_with_token(&TOKEN).await {
        println!("Failed to login to Home Assistant, {}", err);
        return AppRuntime {
            is_connected: false,
            conn: &None,}
    }

        AppRuntime {
            is_connected: true,
            conn: &Some(conn),}
    }
}
impl AppRuntime {
    pub fn print(&self) {
        print!("AppRuntime")
    }
}

lazy_static!(
    pub static ref APP_RUNTIME: AppRuntime = AppRuntime::start("localhost", 8123, false).await;
);
// #[macro_export]
// macro_rules! app_runtime {
//     ($host:literal, $port:literal, $ssl:literal) => {
//         // AppRuntime::start($host, $port, $ssl);
//         println!("{}{}{}", $host, $port, $ssl);
//     };
// }
//
// app_runtime!("localhost", 8123, false);
