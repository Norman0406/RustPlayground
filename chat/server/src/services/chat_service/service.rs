use crate::util;
use chat::chat_service_server;
use chat::*;
use futures::channel::oneshot;
use proto::chat;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tonic::{Request, Response, Status};
use super::user_list::UserList;

pub struct ChatService {
    users: Arc<Mutex<UserList>>,
}

impl ChatService {
    pub fn new() -> chat_service_server::ChatServiceServer<ChatService> {
        chat_service_server::ChatServiceServer::new(ChatService {
            users: Arc::new(Mutex::new(UserList::new()))
        })
    }
}

#[tonic::async_trait]
impl chat_service_server::ChatService for ChatService {
    type ReceiveStream = util::DropReceiver<Result<ReceiveResponse, Status>>;

    async fn send(&self, request: Request<SendRequest>) -> Result<Response<SendResponse>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let reply = chat::SendResponse {
            message_id: Some(chat::MessageId {
                id: String::from("Hallo"),
            }),
        };

        Ok(Response::new(reply))
    }

    async fn receive(
        &self,
        request: Request<ReceiveRequest>,
    ) -> Result<Response<Self::ReceiveStream>, Status> {
        let user_id = match request.metadata().get("user_id") {
            Some(user_id) => user_id.to_str(),
            None => return Err(Status::unauthenticated("no user id found")),
        };

        let user_id = match user_id {
            Ok(user_id) => user_id,
            Err(err) => return Err(Status::internal(err.to_string())),
        };

        {
            let mut users = match self.users.lock() {
                Ok(guard) => guard,
                Err(_) => return Err(Status::internal("unable to acquire lock")),
            };
            match users.add_user(user_id) {
                Ok(()) => {}
                Err(e) => return Err(Status::internal(e)),
            }
        }

        let (mut tx, rx) = mpsc::channel(4);

        let (finish_tx, finish_rx) = oneshot::channel();

        // create a new receiver that is able to detect when the connection is closed
        let rx = util::DropReceiver::new(rx, finish_tx);

        let users = self.users.clone();
        let user_id = String::from(user_id);

        tokio::spawn(async move {
            let response = ReceiveResponse {
                notification: Some(IncomingNotification {
                    from: Some(User {
                        id: String::from("test"),
                    }),
                    types: None,
                }),
            };

            tx.send(Ok(response)).await.unwrap();

            // wait until stream is finished
            finish_rx.await.unwrap();

            // remove user from internal list
            let remove_user_result: Result<(), String>;
            {
                let mut users = users.lock().unwrap();
                remove_user_result = users.remove_user(user_id.as_str());
            }
            match remove_user_result {
                Ok(()) => {}
                Err(e) => {
                    tx.send(Err(Status::internal(format!("Error removing user: {}", e))))
                        .await
                        .unwrap();
                }
            };
        });

        Ok(Response::new(rx))
    }
}
