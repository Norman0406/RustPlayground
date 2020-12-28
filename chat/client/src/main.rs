use chat::chat_service_client::ChatServiceClient;
use chat::{ReceiveRequest, ReceiveResponse, SendRequest, SendResponse};
use proto::chat;
use std::io::Write;
use std::sync::mpsc;
use tonic::{transport::Endpoint, Request};

async fn connect(
    clients: mpsc::Sender<Option<ChatServiceClient<tonic::transport::Channel>>>,
    user_id: String,
) {
    let endpoint = Endpoint::from_static("http://localhost:50051");

    let channel: tonic::transport::Channel;
    loop {
        channel = match endpoint.connect().await {
            Ok(channel) => channel,
            Err(_error) => continue,
        };

        break;
    }

    println!("Connected to {}", endpoint.uri());

    // create a client and append the user id to the metadata
    let client = ChatServiceClient::with_interceptor(channel, move |mut req: Request<()>| {
        req.metadata_mut().insert(
            "user_id",
            tonic::metadata::MetadataValue::from_str(user_id.as_str()).unwrap(),
        );

        Ok(req)
    });
    clients.send(Some(client)).unwrap();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print!("User-ID: ");
    std::io::stdout().flush()?;

    let mut user_id = String::new();
    std::io::stdin().read_line(&mut user_id)?;

    let (sender, receiver) = mpsc::channel();

    tokio::spawn(async {
        connect(sender, user_id).await;
    });

    loop {
        if let Some(mut client) = receiver.recv().unwrap() {
            let mut receive_stream = client
                .receive(Request::new(ReceiveRequest {}))
                .await?
                .into_inner();

            while let Some(response) = receive_stream.message().await? {
                println!(
                    "Notification received from {}",
                    response.notification.unwrap().from.unwrap().id
                );
            }
        }

        break;
    }

    Ok(())
}
