use tonic::{Request, Response, Status};
use proto::hello;
use hello::greeter_server;
use hello::{HelloReply, HelloRequest};

#[derive(Default)]
pub struct Greeter {
}

impl Greeter {
    pub fn new() -> greeter_server::GreeterServer<Greeter> {
        greeter_server::GreeterServer::new(Greeter::default())
    }
}

#[tonic::async_trait]
impl greeter_server::Greeter for Greeter {
    async fn say_hello(&self, request: Request<HelloRequest>) -> Result<Response<HelloReply>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let reply = hello::HelloReply {
            message: format!("Hello {}!", request.into_inner().name),
        };

        Ok(Response::new(reply))
    }
}
