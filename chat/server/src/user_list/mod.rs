mod user;
mod user_data;

use proto::chat;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tonic::{Request, Status};
use user::User;
use user_data::UserData;

pub struct UserList {
    users: Vec<User>,
}

impl UserList {
    pub fn new() -> UserList {
        UserList { users: vec![] }
    }

    pub fn create_user(&mut self, user_id: &str) -> Result<UserData, &str> {
        // check if user exists
        if self.users.iter().position(|v| v.id() == user_id).is_some() {
            return Err("User already exists");
        }

        let mut user = User::new(user_id);

        for other_user in &mut self.users {
            // notify other users that this user is online
            let send_result = other_user
                .user_data
                .sender()
                .try_send(chat::IncomingNotification {
                    from: Some(user.user_data.user()),
                    types: Some(chat::incoming_notification::Types::Online(
                        chat::incoming_notification::Online { is_online: true },
                    )),
                });

            if send_result.is_err() {
                println!("Could not send online notification to user {}", user.id());
            }

            // notify the new user of all currently active users
            let send_result = user
                .user_data
                .sender()
                .try_send(chat::IncomingNotification {
                    from: Some(other_user.user_data.user()),
                    types: Some(chat::incoming_notification::Types::Online(
                        chat::incoming_notification::Online { is_online: true },
                    )),
                });

            if send_result.is_err() {
                println!("Could not send online notification to user {}", user.id());
            }
        }

        let user_data = user.user_data.clone();

        self.users.push(user);

        Ok(user_data)
    }

    pub fn remove_user(&mut self, user_id: &str) -> Result<(), String> {
        let mut user = match self.users.iter().position(|v| v.id() == user_id) {
            Some(index) => self.users.remove(index),
            None => return Err(String::from("user id not found")),
        };

        // set user as offline and notify other users
        self.set_user_online(&mut user, false);

        Ok(())
    }

    pub fn set_user_online(&mut self, user: &mut User, is_online: bool) {
        user.user_data.set_online(is_online);

        let user_data = user.user_data.clone();

        for other_user in &mut self.users {
            // don't send this notification to the current user
            if other_user.id() == user.id() {
                continue;
            }

            let send_result = other_user
                .user_data
                .sender()
                .try_send(chat::IncomingNotification {
                    from: Some(user_data.user()),
                    types: Some(chat::incoming_notification::Types::Online(
                        chat::incoming_notification::Online {
                            is_online: is_online,
                        },
                    )),
                });

            if send_result.is_err() {
                eprintln!(
                    "Could not send offline notification to user {}",
                    user_data.id()
                );
            }
        }
    }

    pub fn get_user(&self, user_id: &str) -> Result<&User, String> {
        let user = match self.users.iter().position(|v| v.id() == user_id) {
            Some(index) => &self.users[index],
            None => return Err(String::from("user id not found")),
        };

        Ok(user)
    }

    fn get_user_mut(&mut self, user_id: &str) -> Result<&mut User, String> {
        let user = match self.users.iter().position(|v| v.id() == user_id) {
            Some(index) => &mut self.users[index],
            None => return Err(String::from("user id not found")),
        };

        Ok(user)
    }

    fn get_user_id_and_token_from_request<T>(
        request: &Request<T>,
    ) -> Result<(String, String), String> {
        let user_id = match request.metadata().get("user_id") {
            Some(user_id) => user_id.to_str(),
            None => return Err(String::from("no user id in metadata")),
        };

        let user_id = match user_id {
            Ok(user_id) => user_id,
            Err(err) => return Err(err.to_string()),
        };

        let user_token = match request.metadata().get("user_token") {
            Some(user_token) => user_token.to_str(),
            None => return Err(String::from("no user id in metadata")),
        };

        let user_token = match user_token {
            Ok(user_token) => user_token,
            Err(err) => return Err(err.to_string()),
        };

        Ok((String::from(user_id), String::from(user_token)))
    }

    pub fn get_user_from_request<T>(
        request: &Request<T>,
        users: &Arc<Mutex<UserList>>,
    ) -> Result<UserData, String> {
        let (user_id, _user_token) = match UserList::get_user_id_and_token_from_request(&request) {
            Ok((user_id, user_token)) => (user_id, user_token),
            Err(err) => return Err(err),
        };

        let users = match users.lock() {
            Ok(guard) => guard,
            Err(_) => return Err(String::from("unable to acquire lock")),
        };

        let user = users.get_user(&user_id)?;
        Ok(user.user_data.clone())
    }

    fn is_user_authenticated(&self, user_id: &str, user_token: &str) -> bool {
        match self
            .users
            .iter()
            .position(|v| v.id() == user_id && v.user_data.token() == user_token)
        {
            Some(_index) => true,
            None => false,
        }
    }

    pub fn authenticate(
        request: Request<()>,
        users: &Arc<Mutex<UserList>>,
    ) -> Result<Request<()>, Status> {
        let (user_id, user_token) = match UserList::get_user_id_and_token_from_request(&request) {
            Ok((user_id, user_token)) => (user_id, user_token),
            Err(_err) => return Err(Status::unauthenticated("could not authenticate")),
        };

        let users = match users.lock() {
            Ok(guard) => guard,
            Err(_) => return Err(Status::unauthenticated("unable to acquire lock")),
        };

        // check if user is authenticated
        match users.is_user_authenticated(&user_id, &user_token) {
            true => Ok(request),
            false => Err(Status::unauthenticated("could not authenticate")),
        }
    }

    pub fn take_receiver<T>(
        request: &Request<T>,
        users: &Arc<Mutex<UserList>>,
    ) -> Result<mpsc::Receiver<chat::IncomingNotification>, String> {
        let (user_id, _user_token) = match UserList::get_user_id_and_token_from_request(&request) {
            Ok((user_id, user_token)) => (user_id, user_token),
            Err(err) => return Err(err),
        };

        let mut users = match users.lock() {
            Ok(guard) => guard,
            Err(_) => return Err(String::from("unable to acquire lock")),
        };

        let user = users.get_user_mut(&user_id)?;
        Ok(user.take_receiver())
    }
}
