use crate::util;
use crate::UserList;
use chat::authentication_service_server;
use chat::*;
use futures::channel::oneshot;
use proto::chat;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tonic::{Request, Response, Status};

pub struct AuthenticationService {
    users: Arc<Mutex<UserList>>,
}

impl AuthenticationService {
    pub fn new(
        users: Arc<Mutex<UserList>>,
    ) -> authentication_service_server::AuthenticationServiceServer<AuthenticationService> {
        let service = AuthenticationService { users: users };

        authentication_service_server::AuthenticationServiceServer::new(service)
    }
}

#[tonic::async_trait]
impl authentication_service_server::AuthenticationService for AuthenticationService {
    type AuthenticateStream = util::ResponseStream<Result<AuthenticateResponse, Status>>;

    async fn authenticate(
        &self,
        request: Request<AuthenticateRequest>,
    ) -> Result<Response<Self::AuthenticateStream>, Status> {
        let request = request.into_inner();

        // create user
        let user;
        {
            let mut users = match self.users.lock() {
                Ok(guard) => guard,
                Err(_) => return Err(Status::internal("unable to acquire lock")),
            };

            user = users.create_user(&request.name).unwrap();
        }

        let (finish_tx, finish_rx) = oneshot::channel();
        let (mut stream_tx, stream_rx) = mpsc::channel(4);

        // report new user id back to caller
        let response = Ok(AuthenticateResponse { id: user.id() });
        stream_tx.try_send(response).unwrap();

        let users = self.users.clone();

        tokio::spawn(async move {
            // wait until stream is finished
            finish_rx.await.unwrap();

            // remove user from internal list
            let remove_user_result;
            {
                let mut users = users.lock().unwrap();
                remove_user_result = users.remove_user(user.id().as_str());
            }

            match remove_user_result {
                Ok(()) => {}
                Err(e) => {
                    eprintln!("Error removing user: {}", e);
                }
            };
        });

        let response_stream =
            util::ResponseStream::new_with_close_notification(finish_tx, stream_rx);

        Ok(Response::new(response_stream))
    }
}
