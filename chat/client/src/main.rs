use chat::authentication_service_client::AuthenticationServiceClient;
use chat::chat_service_client::ChatServiceClient;
use chat::{AuthenticateRequest, AuthenticateResponse};
use chat::{ReceiveRequest, ReceiveResponse, SendRequest, SendResponse};
use proto::chat;
use std::io::Write;
use std::sync::mpsc;
use tonic::{transport::Endpoint, Request};

async fn connect(
    clients: mpsc::Sender<Option<ChatServiceClient<tonic::transport::Channel>>>,
    user_name: String,
) {
    let endpoint = Endpoint::from_static("http://localhost:50001");

    // let channel: tonic::transport::Channel;
    // loop {
    //     channel = match endpoint.connect().await {
    //         Ok(channel) => channel,
    //         Err(_error) => continue,
    //     };

    //     break;
    // }

    let channel = match endpoint.connect().await {
        Ok(channel) => channel,
        Err(_error) => {
            panic!("Error");
        }
    };

    println!("Connected to {}", endpoint.uri());

    let mut authentication_client = AuthenticationServiceClient::new(channel.clone());

    let mut authenticate_stream = authentication_client
        .authenticate(Request::new(AuthenticateRequest {
            name: user_name.clone(),
        }))
        .await
        .unwrap()
        .into_inner();

    while let Some(response) = authenticate_stream.message().await.unwrap() {
        let user_id = response.id;
        let user_token = response.token;

        println!(
            "Authenticated user {}: id {}, token {}",
            user_name, user_id, user_token
        );

        let chat_client =
            ChatServiceClient::with_interceptor(channel.clone(), move |mut req: Request<()>| {
                req.metadata_mut().insert(
                    "user_id",
                    tonic::metadata::AsciiMetadataValue::from_str(&user_id).unwrap(),
                );
                req.metadata_mut().insert(
                    "user_token",
                    tonic::metadata::AsciiMetadataValue::from_str(&user_token).unwrap(),
                );

                Ok(req)
            });

        clients.send(Some(chat_client)).unwrap();
    }
}

fn get_user_name() -> String {
    print!("Username: ");
    std::io::stdout().flush().unwrap();

    let mut user_name = String::new();
    std::io::stdin().read_line(&mut user_name).unwrap();
    user_name.retain(|c| !c.is_control());

    user_name
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let user_name = get_user_name();

    let (sender, receiver) = mpsc::channel();

    tokio::spawn(async {
        connect(sender, user_name).await;
    });

    loop {
        let client = receiver.recv().unwrap();

        if let Some(mut client) = client {
            let mut receive_stream = client
                .receive(Request::new(ReceiveRequest {}))
                .await?
                .into_inner();

            while let Some(response) = receive_stream.message().await? {
                let notification = response.notification.unwrap();
                let user = notification.from.unwrap();
                let from_user_id = user.id;
                let from_user_name = user.name;

                let notification_type = notification.types.unwrap();
                match notification_type {
                    chat::incoming_notification::Types::Delivered(delivered) => {
                        println!(
                            "Message {} was delivered to user {}",
                            delivered.message_id.unwrap().id,
                            from_user_id
                        );
                    }
                    chat::incoming_notification::Types::Read(read) => {
                        println!(
                            "User {} read message {}",
                            from_user_id,
                            read.message_id.unwrap().id
                        );
                    }
                    chat::incoming_notification::Types::Typing(typing) => {
                        let typing_nottyping = match typing.is_typing {
                            true => "typing",
                            false => "not typing",
                        };

                        println!("User {} is {}", from_user_id, typing_nottyping);
                    }
                    chat::incoming_notification::Types::Online(online) => {
                        let online_offline = match online.is_online {
                            true => "online",
                            false => "offline",
                        };

                        println!("User {} is {}", from_user_name, online_offline);
                    }
                    chat::incoming_notification::Types::Message(message) => {
                        println!(
                            "Message from user {}: {}",
                            from_user_id,
                            message.message_content.unwrap().content
                        );
                    }
                }
            }
        }

        break;
    }

    Ok(())
}
