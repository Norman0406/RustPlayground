mod hello;
mod services;

use tonic::transport::Server;
use futures::prelude::*;

use hello::Greeter;
use services::UnaryService;
use services::ServerStreamingService;
use services::ClientStreamingService;
use services::BidirectionalStreamingService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();

    println!("Server listening on {}", addr);

    let shutdown_signal = tokio::signal::ctrl_c().map(|_| ());

    Server::builder()
        .add_service(Greeter::new())
        .add_service(UnaryService::new())
        .add_service(ServerStreamingService::new())
        .add_service(ClientStreamingService::new())
        .add_service(BidirectionalStreamingService::new())
        .serve_with_shutdown(addr, shutdown_signal)
        .await?;

    println!("Server finished");

    Ok(())
}
