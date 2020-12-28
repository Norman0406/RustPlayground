mod services;
mod util;

use futures::prelude::*;
use structopt::StructOpt;
use tonic::transport::Server;

use services::ChatService;

#[derive(StructOpt)]
#[structopt(about = "A gRPC test server")]
struct Cli {
    #[structopt(short, help = "The port on which the gRPC server will be opened")]
    port: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::from_args();

    let addr = format!("127.0.0.1:{}", args.port).parse().unwrap();

    println!("Server listening on {}", addr);

    let shutdown_signal = tokio::signal::ctrl_c().map(|_| ());

    Server::builder()
        .add_service(ChatService::new())
        .serve_with_shutdown(addr, shutdown_signal)
        .await?;

    println!("Server finished");

    Ok(())
}
