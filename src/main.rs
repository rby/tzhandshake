use std::path::PathBuf;

use clap::Parser;
use rand::thread_rng;
use tzhandhsake::{identity::Identity, p2p::handshake::Handshake};

use anyhow::Result;
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

    #[arg(short, long, required = true)]
    identity_path: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    println!("connecting to {}", args.node);
    let mut rng = thread_rng();
    let _chan = Handshake::identity(Identity::from_file(args.identity_path)?)
        .generate_nonce(&mut rng)
        .connect(args.node)
        .await?;

    println!("end of handshake");

    Ok(())
}
