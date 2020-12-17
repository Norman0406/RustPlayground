use tonic::{Request, Response, Status};
use tokio::sync::mpsc;
use proto::services;
use services::server_streaming_service_server;
use services::{ServerStreamingRequest, ServerStreamingResponse};

#[derive(Default)]
pub struct ServerStreamingService {
}

impl ServerStreamingService {
    pub fn new() -> server_streaming_service_server::ServerStreamingServiceServer<ServerStreamingService> {
        server_streaming_service_server::ServerStreamingServiceServer::new(ServerStreamingService::default())
    }
}

#[tonic::async_trait]
impl server_streaming_service_server::ServerStreamingService for ServerStreamingService {
    type ServerStreamStream = mpsc::Receiver<Result<ServerStreamingResponse, Status>>;

    async fn server_stream(&self, request: Request<ServerStreamingRequest>) -> Result<Response<Self::ServerStreamStream>, Status> {
        let request = request.into_inner();

        let (mut tx, rx) = mpsc::channel(4);

        tokio::spawn(async move {
            for n in 0..request.number_of_responses {
                let response = services::ServerStreamingResponse {
                    greeting: format!("Hello ({}) {}", n, request.name)
                };
    
                tx.send(Ok(response)).await.unwrap();
            }

            println!(" /// done sending");
        });

        Ok(Response::new(rx))
    }
}
