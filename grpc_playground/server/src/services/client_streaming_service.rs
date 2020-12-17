use tonic::{Request, Response, Streaming, Status};
use proto::services;
use services::client_streaming_service_server;
use services::{ClientStreamingRequest, ClientStreamingResponse};

#[derive(Default)]
pub struct ClientStreamingService {
}

impl ClientStreamingService {
    pub fn new() -> client_streaming_service_server::ClientStreamingServiceServer<ClientStreamingService> {
        client_streaming_service_server::ClientStreamingServiceServer::new(ClientStreamingService::default())
    }
}

#[tonic::async_trait]
impl client_streaming_service_server::ClientStreamingService for ClientStreamingService {
    async fn client_stream(&self, request: Request<Streaming<ClientStreamingRequest>>) -> Result<Response<ClientStreamingResponse>, Status> {

        let mut stream = request.into_inner();

        let mut list_of_names = vec!();
        while let Some(request) = stream.message().await? {
            list_of_names.push(request.name);
        }

        let reply = services::ClientStreamingResponse {
            greeting: format!("Hello {}", list_of_names.join(", "))
        };

        Ok(Response::new(reply))
    }
}
