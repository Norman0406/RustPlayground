use tonic::{Request, Response, Streaming, Status};
use tokio::sync::mpsc;
use proto::services;
use services::bidirectional_streaming_service_server;
use services::{BidirectionalStreamingRequest, BidirectionalStreamingResponse};

#[derive(Default)]
pub struct BidirectionalStreamingService {
}

impl BidirectionalStreamingService {
    pub fn new() -> bidirectional_streaming_service_server::BidirectionalStreamingServiceServer<BidirectionalStreamingService> {
        bidirectional_streaming_service_server::BidirectionalStreamingServiceServer::new(BidirectionalStreamingService::default())
    }
}

#[tonic::async_trait]
impl bidirectional_streaming_service_server::BidirectionalStreamingService for BidirectionalStreamingService {
    type BidirectionalStreamStream = mpsc::Receiver<Result<BidirectionalStreamingResponse, Status>>;

    async fn bidirectional_stream(&self, request: Request<Streaming<BidirectionalStreamingRequest>>) -> Result<Response<Self::BidirectionalStreamStream>, Status> {
        let (mut tx, rx) = mpsc::channel(4);

        let mut stream = request.into_inner();

        while let Some(request) = stream.message().await? {
            println!("received request");

            let response = services::BidirectionalStreamingResponse {
                greeting: format!("Hello {}!", request.name),
            };

            tx.send(Ok(response)).await.unwrap();

            println!(" /// done sending");
        }

        Ok(Response::new(rx))
    }
}
