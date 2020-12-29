use super::user_list::UserList;
use crate::util;
use chat::chat_service_server;
use chat::*;
use futures::channel::oneshot;
use futures::stream::StreamExt;
use proto::chat;
use std::sync::{Arc, Mutex};
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub struct ChatService {
    users: Arc<Mutex<UserList>>,
}

impl ChatService {
    pub fn new() -> chat_service_server::ChatServiceServer<ChatService> {
        chat_service_server::ChatServiceServer::new(ChatService {
            users: Arc::new(Mutex::new(UserList::new())),
        })
    }

    fn authenticate_call<T>(&self, request: &Request<T>) -> Result<(), Status> {
        // get user id from metadata
        let user_id = match request.metadata().get("user_id") {
            Some(user_id) => user_id.to_str(),
            None => return Err(Status::unauthenticated("no user id in metadata")),
        };

        let user_id = match user_id {
            Ok(user_id) => user_id,
            Err(err) => return Err(Status::unauthenticated(err.to_string())),
        };

        let users = match self.users.lock() {
            Ok(guard) => guard,
            Err(_) => return Err(Status::unauthenticated("unable to acquire lock")),
        };

        // check if user exists
        match users.get_user(user_id) {
            Ok(_user) => Ok(()),
            Err(_) => return Err(Status::unauthenticated("user not found")),
        }
    }
}

#[tonic::async_trait]
impl chat_service_server::ChatService for ChatService {
    type ReceiveStream = util::ResponseStream<Result<ReceiveResponse, Status>>;

    async fn send(&self, request: Request<SendRequest>) -> Result<Response<SendResponse>, Status> {
        self.authenticate_call(&request)?;

        let request = request.into_inner();
        let notification = request.notification.unwrap();

        let to_user_id = notification.to.unwrap().id;

        // get the receiving user
        let (to_user, mut to_user_sender);
        {
            let users = match self.users.lock() {
                Ok(guard) => guard,
                Err(_) => return Err(Status::internal("unable to acquire lock")),
            };

            let to_user_ref = match users.get_user(to_user_id.as_str()) {
                Ok(user) => user,
                Err(e) => return Err(Status::internal(e)),
            };

            to_user = to_user_ref.user.clone();
            to_user_sender = to_user_ref.notifications_tx.clone();
        }

        // create a default reply
        let mut reply = chat::SendResponse { message_id: None };

        let notification_type = notification.types.unwrap();
        let mut incoming_notification = None;
        match notification_type {
            chat::outgoing_notification::Types::Typing(_typing) => {
                // TODO
            }
            chat::outgoing_notification::Types::Read(_read) => {
                // TODO
            }
            chat::outgoing_notification::Types::Message(_message) => {
                incoming_notification = Some(chat::IncomingNotification {
                    from: Some(to_user),
                    types: None,
                });

                // TODO: enqueue message id somewhere
                let message_id = Uuid::new_v5(&Uuid::NAMESPACE_OID, "chat".as_bytes());

                // return the message id of this message
                reply.message_id = Some(chat::MessageId {
                    id: message_id.to_hyphenated().to_string(),
                });
            }
        }

        // send notification to receiving user
        if incoming_notification.is_none()
            || to_user_sender
                .try_send(incoming_notification.unwrap())
                .is_err()
        {
            return Err(Status::internal("Could not send notification"));
        }

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

        let notifications_rx;
        {
            let mut users = match self.users.lock() {
                Ok(guard) => guard,
                Err(_) => return Err(Status::internal("unable to acquire lock")),
            };
            notifications_rx = match users.create_user(user_id) {
                Ok(rx) => rx,
                Err(e) => return Err(Status::internal(e)),
            }
        }

        let (finish_tx, finish_rx) = oneshot::channel();

        let users = self.users.clone();
        let user_id = String::from(user_id);

        tokio::spawn(async move {
            // wait until stream is finished
            finish_rx.await.unwrap();

            // remove user from internal list
            let remove_user_result;
            {
                let mut users = users.lock().unwrap();
                remove_user_result = users.remove_user(user_id.as_str());
            }

            match remove_user_result {
                Ok(()) => {}
                Err(e) => {
                    eprintln!("Error removing user: {}", e);
                }
            };
        });

        let response_stream = util::ResponseStream::new_with_close_notification(
            finish_tx,
            notifications_rx.map(|notification| {
                Ok(ReceiveResponse {
                    notification: Some(notification),
                })
            }),
        );

        Ok(Response::new(response_stream))
    }
}
