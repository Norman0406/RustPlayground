use tonic::{Request, Response, Status};
use proto::services;
use services::unary_service_server;
use services::{UnaryCallRequest, UnaryCallResponse};

#[derive(Default)]
pub struct UnaryService {
}

impl UnaryService {
    pub fn new() -> unary_service_server::UnaryServiceServer<UnaryService> {
        unary_service_server::UnaryServiceServer::new(UnaryService::default())
    }
}

#[tonic::async_trait]
impl unary_service_server::UnaryService for UnaryService {
    async fn unary_call(&self, request: Request<UnaryCallRequest>) -> Result<Response<UnaryCallResponse>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let reply = services::UnaryCallResponse {
            greeting: format!("Hello {}!", request.into_inner().name),
        };

        Ok(Response::new(reply))
    }
}
