use clap::Parser;
use orderly::orderly;

/// Pulls order depths for the given currency pair from the WebSocket feeds of multiple exchanges.
/// Publishes a merged order book as a gRPC stream.
#[derive(Parser)]
struct Cli {
    #[clap(short, long, help = "(Optional) Currency pair to subscribe to. Default: ETH/BTC")]
    symbol: Option<String>,

    #[clap(short, long, help = "(Optional) Port number on which the the gRPC server will be hosted. Default: 50051")]
    port: Option<usize>,

    #[clap(long, help = "(Optional) Disable Bitstamp. Default: false")]
    no_bitstamp: bool,

    #[clap(long, help = "(Optional) Disable Binance. Default: false")]
    no_binance: bool,

    #[clap(long, help = "(Optional) Disable Kraken. Default: false")]
    no_kraken: bool,

    #[clap(long, help = "(Optional) Disable Coinbase. Default: false")]
    no_coinbase: bool,

    #[clap(long, help = "(Optional) Disable Gateio. Default: false")]
    no_gateio: bool,

}

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Cli::parse();
    let symbol: String = args.symbol.unwrap_or("BTC/USDT".to_string());
    let port: usize = args.port.unwrap_or(50054);
    let no_bitstamp: bool = args.no_bitstamp;
    let no_binance: bool = args.no_binance;
    let no_kraken: bool = args.no_kraken;
    let no_coinbase: bool = args.no_coinbase;
    let no_gateio: bool = args.no_gateio;

    orderly::run(&symbol, port,
                 no_bitstamp, no_binance, no_kraken, no_coinbase, no_gateio).await.unwrap();
}

