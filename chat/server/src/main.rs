mod services;
mod user_list;
mod util;

use futures::prelude::*;
use std::sync::{Arc, Mutex};
use structopt::StructOpt;
use tonic::transport::Server;
use user_list::UserList;

use services::AuthenticationService;
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

    let users = Arc::new(Mutex::new(UserList::new()));

    Server::builder()
        .add_service(AuthenticationService::new(users.clone()))
        .add_service(ChatService::new(users))
        .serve_with_shutdown(addr, shutdown_signal)
        .await?;

    println!("Server finished");

    Ok(())
}
