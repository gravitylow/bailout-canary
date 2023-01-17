use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use sha256::digest;
use url::Url;
use tungstenite::{connect, Message};
use lambda_runtime::{handler_fn};
use serde::{Serialize, Deserialize};
use serde_json::{Value};
use log::{info, warn};

const CLIENT_PLATFORM: &'static str = "canary";
const CLIENT_VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn get_api_key() -> String {
    env::var("BAILOUT_API_KEY").unwrap()
}

fn get_signed_url() -> String {
    let timestamp = current_time();
    let client_id = Uuid::new_v4().to_string();
    let api_key = get_api_key();
    let unsigned_id = get_identifier(&client_id, &timestamp, api_key);
    info!("Signing {}", unsigned_id);
    let auth_signature = digest(unsigned_id);
    info!("Got digest: {}", auth_signature);
    get_url(&client_id, CLIENT_PLATFORM, CLIENT_VERSION, &timestamp, &auth_signature)
}

fn get_url(client_id: &str, client_platform: &str, client_version: &str, auth_timestamp: &str, auth_signature: &str) -> String {
    format!(
        "wss://atlas.bailoutapp.io/api/v1/\
        ?client_id={client_id}\
        &client_platform={client_platform}\
        &client_version={client_version}\
        &auth_timestamp={auth_timestamp}\
        &auth_signature={auth_signature}",
        client_id = client_id,
        client_platform = client_platform,
        client_version = client_version,
        auth_timestamp = auth_timestamp,
        auth_signature = auth_signature,
    )
}

fn get_identifier(client_id: &str, auth_timestamp: &str, api_key: String) -> String {
    format!(
        "connect:{client_id}:{auth_timestamp}:{api_key}",
        client_id = client_id,
        auth_timestamp = auth_timestamp,
        api_key = api_key,
    )
}

fn current_time() -> String {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_secs().to_string()
}


#[derive(Debug, Deserialize)]
struct Request {
    pub body: String,
}

#[derive(Debug, Serialize)]
struct SuccessResponse {
    pub body: String,
}

#[derive(Debug, Serialize)]
struct FailureResponse {
    pub body: String,
}

// Implement Display for the Failure response so that we can then implement Error.
impl std::fmt::Display for FailureResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.body)
    }
}

// Implement Error for the FailureResponse so that we can `?` (try) the Response
// returned by `lambda_runtime::run(func).await` in `fn main`.
impl std::error::Error for FailureResponse {}

type Response = Result<SuccessResponse, FailureResponse>;

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    // let func = handler_fn(handler);
    // lambda_runtime::run(func).await?;
    // for local testing:
    handler().await?;
    Ok(())
}

async fn handler() -> Response {
    match simple_logger::init_with_level(log::Level::Info) {
        Err(e) => warn!("{:?}", e),
        _ => ()
    }
    // info!("Handling a request: {:?}", req);
    let url = get_signed_url();
	info!("Connecting to {}", url);
	
    let (mut socket, _response) = connect(
        Url::parse(&url).unwrap()
    ).expect("Can't connect");
    info!("Connected!");

    loop {
        match socket.read_message().expect("Error reading from socket") {
            msg @ Message::Text(_) => {
                let data: Value = serde_json::from_str(&msg.into_text().expect("should be text")).expect("Error parsing JSON");
                info!("Received: {}", data);
                assert_eq!("connect", data["type"]);
                assert_eq!(210, data["status"]);
                assert_eq!("Connected", data["message"]);
                return Ok(SuccessResponse {
                    body: "Successfully connected".to_string()
                });
            }
            Message::Binary(_) | Message::Ping(_) | Message::Pong(_) | Message::Close(_) | Message::Frame(_) => {}
        }
    }

    // TODO start a game
    // socket.write_message(Message::Text(r#"{
    //     "action": "authenticate",
    //     "data": {
    //         "key_id": "API-KEY",
    //         "secret_key": "SECRET-KEY"
    //     }
    // }"#.into()));
}
