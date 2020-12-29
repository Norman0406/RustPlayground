use proto::chat;
use tokio::sync::mpsc;

pub struct User {
    pub user: chat::User,
    pub notifications_tx: mpsc::Sender<chat::IncomingNotification>,
}

impl User {
    fn new(user_id: &str) -> (User, mpsc::Receiver<chat::IncomingNotification>) {
        let (notifications_tx, notifications_rx) = mpsc::channel(4);

        let user = User {
            user: chat::User {
                id: String::from(user_id),
            },
            notifications_tx: notifications_tx,
        };

        (user, notifications_rx)
    }
}

pub struct UserList {
    users: Vec<User>,
}

impl UserList {
    pub fn new() -> UserList {
        UserList { users: vec![] }
    }

    pub fn create_user(
        &mut self,
        user_id: &str,
    ) -> Result<mpsc::Receiver<chat::IncomingNotification>, &str> {
        // check if user exists
        match self.users.iter().position(|v| v.user.id == user_id) {
            Some(_index) => return Err("User already exists"),
            None => (),
        };

        let (mut user, notifications_rx) = User::new(user_id);

        for other_user in &mut self.users {
            // notify other users that this user is online
            let send_result = other_user
                .notifications_tx
                .try_send(chat::IncomingNotification {
                    from: Some(user.user.clone()),
                    types: Some(chat::incoming_notification::Types::Online(
                        chat::incoming_notification::Online { is_online: true },
                    )),
                });

            if send_result.is_err() {
                println!(
                    "Could not send online notification to user {}",
                    user.user.id
                );
            }

            // notify the new user of all currently active users
            let send_result = user.notifications_tx.try_send(chat::IncomingNotification {
                from: Some(other_user.user.clone()),
                types: Some(chat::incoming_notification::Types::Online(
                    chat::incoming_notification::Online { is_online: true },
                )),
            });

            if send_result.is_err() {
                println!(
                    "Could not send online notification to user {}",
                    user.user.id
                );
            }
        }

        self.users.push(user);

        Ok(notifications_rx)
    }

    pub fn remove_user(&mut self, user_id: &str) -> Result<(), String> {
        let user = match self.users.iter().position(|v| v.user.id == user_id) {
            Some(index) => self.users.remove(index),
            None => return Err(String::from("user id not found")),
        };

        // notify other users that this user is not online anymore
        for other_user in &mut self.users {
            let send_result = other_user
                .notifications_tx
                .try_send(chat::IncomingNotification {
                    from: Some(user.user.clone()),
                    types: Some(chat::incoming_notification::Types::Online(
                        chat::incoming_notification::Online { is_online: false },
                    )),
                });

            if send_result.is_err() {
                eprintln!(
                    "Could not send offline notification to user {}",
                    user.user.id
                );
            }
        }

        Ok(())
    }

    pub fn get_user(&self, user_id: &str) -> Result<&User, &str> {
        let user = match self.users.iter().position(|v| v.user.id == user_id) {
            Some(index) => &self.users[index],
            None => return Err("user id not found"),
        };

        Ok(user)
    }
}
