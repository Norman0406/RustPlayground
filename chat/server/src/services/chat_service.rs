use crate::util;
use crate::UserList;
use chat::chat_service_server;
use chat::*;
use futures::stream::StreamExt;
use proto::chat;
use std::sync::{Arc, Mutex};
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub struct ChatService {
    users: Arc<Mutex<UserList>>,
}

impl ChatService {
    pub fn new(users: Arc<Mutex<UserList>>) -> chat_service_server::ChatServiceServer<ChatService> {
        let service = ChatService { users: users };

        let check_auth;
        {
            let users = service.users.clone();
            check_auth = move |request: Request<()>| -> Result<Request<()>, Status> {
                UserList::authenticate(request, &users)
            };
        }

        chat_service_server::ChatServiceServer::with_interceptor(service, check_auth)
    }
}

#[tonic::async_trait]
impl chat_service_server::ChatService for ChatService {
    type ReceiveStream = util::ResponseStream<Result<ReceiveResponse, Status>>;

    async fn send(&self, request: Request<SendRequest>) -> Result<Response<SendResponse>, Status> {
        let user = match UserList::get_user_from_request(&request, &self.users) {
            Ok(user) => user,
            Err(err) => return Err(Status::internal(err)),
        };

        let request = request.into_inner();
        let notification = match request.notification {
            Some(notification) => notification,
            None => return Err(Status::invalid_argument("request.notification is invalid")),
        };

        let to_user = match notification.to {
            Some(user) => user,
            None => {
                return Err(Status::invalid_argument(
                    "request.notification.to is invalid",
                ))
            }
        };

        // get the receiving user
        let mut to_user_sender;
        {
            let users = match self.users.lock() {
                Ok(guard) => guard,
                Err(_) => return Err(Status::internal("unable to acquire lock")),
            };

            to_user_sender = match users.get_user(to_user.id.as_str()) {
                Ok(user) => user.user_data.sender(),
                Err(e) => return Err(Status::internal(e)),
            };
        }

        // create a default reply
        let mut reply = chat::SendResponse { message_id: None };

        let notification_type = match notification.types {
            Some(notification_type) => notification_type,
            None => {
                return Err(Status::invalid_argument(
                    "request.notification.types is invalid",
                ))
            }
        };

        let mut incoming_notification = None;
        match notification_type {
            chat::outgoing_notification::Types::Typing(_typing) => {
                // TODO
            }
            chat::outgoing_notification::Types::Read(_read) => {
                // TODO
            }
            chat::outgoing_notification::Types::Message(message) => {
                // TODO: enqueue message id somewhere
                let message_id = Uuid::new_v4();
                let message_id_string = message_id.to_hyphenated().to_string();

                incoming_notification = Some(chat::IncomingNotification {
                    from: Some(user.user()),
                    types: Some(chat::incoming_notification::Types::Message(
                        chat::incoming_notification::Message {
                            message_id: Some(chat::MessageId {
                                id: message_id_string.clone(),
                            }),
                            message_content: Some(message),
                        },
                    )),
                });

                // return the message id of this message
                reply.message_id = Some(chat::MessageId {
                    id: message_id_string,
                });
            }
        }

        let incoming_notification = match incoming_notification {
            Some(notification) => notification,
            None => return Err(Status::internal("notification could not be created")),
        };

        // send notification to receiving user
        if to_user_sender.try_send(incoming_notification).is_err() {
            return Err(Status::internal("Could not send notification"));
        }

        Ok(Response::new(reply))
    }

    async fn receive(
        &self,
        request: Request<ReceiveRequest>,
    ) -> Result<Response<Self::ReceiveStream>, Status> {
        let notifications_rx = match UserList::take_receiver(&request, &self.users) {
            Ok(rx) => rx,
            Err(err) => return Err(Status::internal(err)),
        };

        let response_stream = util::ResponseStream::new(notifications_rx.map(|notification| {
            Ok(ReceiveResponse {
                notification: Some(notification),
            })
        }));

        Ok(Response::new(response_stream))
    }
}
