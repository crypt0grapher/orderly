use futures::SinkExt;
use crate::error::Error;
use crate::orderbook::{self, Exchange, InTick, ToLevel, ToLevels, ToTick};
use crate::websocket;
use log::{debug, info};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tungstenite::Message;

const GATEIO_WS_URL: &str = "wss://fx-ws.gateio.ws/v4/ws";

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "event", rename_all = "lowercase")]
enum Event {
    All {
        time: u64,
        time_ms: u64,
        channel: String,
        result: GateioSubResult,
    },

    Subscribe {
        time: i64,
        channel: Channel,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        payload: Option<[String; 3]>
    },

    Error { label: String, message: String },

    Response {
        time: u64,
        time_ms: u64,
        channel: String,
        result: GateioSubResult,
    },
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct GateioCommandResponse {
    time: u64,
    time_ms: u64,
    channel: String,
    event: String,
    result: GateioSubResult,
}


#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct GateioSubResult {
    #[serde(rename = "t")]
    time: u64,
    contract: String,
    id: u64,
    #[serde(rename = "bids")]
    bids: Vec<Level>,
    #[serde(rename = "asks")]
    asks: Vec<Level>
}
// #[derive(Debug, Deserialize, Serialize, PartialEq)]
// struct GateioSubResult {
//     #[serde(rename = "t")]
//     time: u64,
//     #[serde(rename = "s")]
//     contract_name: String,
//     #[serde(rename = "U")]
//     first_update_id: u64,
//     #[serde(rename = "u")]
//     last_update_id: u64,
//     #[serde(rename = "b")]
//     changed_bids: Vec<Level>,
//     #[serde(rename = "a")]
//     changed_asks: Vec<Level>,
// }

impl ToTick for Event {
    /// Converts the `Event` into a `Option<InTick>`. Only keep the top ten levels of bids and asks.
    fn maybe_to_tick(&self) -> Option<InTick> {
        match self {
            Event::All { result, .. } => {
                let bids = result.bids.to_levels(orderbook::Side::Bid, 10);
                let asks = result.asks.to_levels(orderbook::Side::Bid, 10);
                Some(InTick { exchange: Exchange::Gateio, bids, asks })
            }
            _ => None,
        }
    }
}


#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct InSubscription {}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct InError {
    code: Option<String>,
    message: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
struct Level {
    #[serde(rename = "p")]
    price: Decimal,
    #[serde(rename = "s")]
    amount: Decimal,
}

impl ToLevel for Level {
    /// Converts a `gateio::Level` into a `orderbook::Level`.
    fn to_level(&self, side: orderbook::Side) -> orderbook::Level {
        orderbook::Level::new(side, self.price, self.amount, Exchange::Gateio)
    }
}

type Channel = String;

pub(crate) async fn connect(symbol: &String) -> Result<websocket::WsStream, Error> {
    let base_currency = symbol.split("/").last().unwrap().to_lowercase();
    let symbol = symbol.to_uppercase().replace("/", "_");
    let url = format!("{}/{}", GATEIO_WS_URL, base_currency);
    let mut ws_stream = websocket::connect(&url).await?;
    subscribe(&mut ws_stream, &symbol).await?;
    Ok(ws_stream)
}

pub(crate) fn parse(msg: Message) -> Result<Option<InTick>, Error> {
    let e = match msg {
        Message::Binary(x) => {
            info!("binary {:?}", x);
            None
        }
        Message::Text(x) => {
            let e = deserialize(x)?;
            debug!("{:?}", e);
            Some(e)
        }
        Message::Ping(x) => {
            info!("Ping {:?}", x);
            None
        }
        Message::Pong(x) => {
            info!("Pong {:?}", x);
            None
        }
        Message::Close(x) => {
            info!("Close {:?}", x);
            None
        }
        Message::Frame(x) => {
            info!("Frame {:?}", x);
            None
        }
    };
    Ok(e.map(|e| e.maybe_to_tick()).flatten())
}

async fn subscribe(
    rx: &mut websocket::WsStream,
    symbol: &String,
) -> Result<(), Error>
{
    let channel = "futures.order_book".to_string();
    let msg = serialize(Event::Subscribe {
        time: chrono::Utc::now().timestamp_millis(),
        channel,
        payload: Some([symbol.clone(), "20".to_string(), "0".to_string()])
        // payload: Some([symbol.clone(), "100ms".to_string(), "20".to_string()])
    })?;
    rx.send(Message::Text(msg)).await?;
    Ok(())
}

fn deserialize(s: String) -> serde_json::Result<Event> {
    Ok(serde_json::from_str(&s)?)
}

fn serialize(e: Event) -> serde_json::Result<String> {
    Ok(serde_json::to_string(&e)?)
}