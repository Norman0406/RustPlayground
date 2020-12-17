use proto::hello;
use hello::greeter_client::GreeterClient;
use hello::HelloRequest;
use tonic::{transport::Endpoint};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = Endpoint::from_static("http://[::1]:50051");

    let channel: tonic::transport::Channel;
    loop {
        channel = match endpoint.connect().await {
            Ok(channel) => channel,
            Err(error) => {
                println!("An error occurred while connection, retrying: {:?}", error);
                continue
            }
        };

        break;
    }

    println!("Connected to {:?}", endpoint.uri());

    let mut client = GreeterClient::new(channel);

    let request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    let response = client.say_hello(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
