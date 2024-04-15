use std::io;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(about = "Handshakes tezos nodes on Ghostnet")]
struct Cli {
    /// Ghostnet not to perform the handshake with
    /// Format "ip:port"
    #[arg(
        short,
        long,
        required = false,
        default_value = "ghostnet.tzinit.org:9732"
    )]
    node: String,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Cli::parse();
    println!("connecting to {}", args.node);
    Ok(())
}
