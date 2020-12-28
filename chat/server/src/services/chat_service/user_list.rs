use proto::chat;
use chat::User;

pub enum UserNotification {
    NewUser(User),
    UserRemoved(User),
}

pub struct UserList {
    pub users: Vec<User>,
    pub notifications_rx: std::sync::mpsc::Receiver<UserNotification>,
    notifications_tx: std::sync::mpsc::Sender<UserNotification>,
}

impl UserList {
    pub fn new() -> UserList {
        let (tx, rx) = std::sync::mpsc::channel();

        UserList {
            users: vec![],
            notifications_rx: rx,
            notifications_tx: tx,
        }
    }

    pub fn add_user(&mut self, user_id: &str) -> Result<(), &str> {
        // check if user exists
        match self.users.iter().position(|v| v.id == user_id) {
            Some(_index) => return Err("User already exists"),
            None => (),
        };

        println!("add user {}", user_id);

        let user = User {
            id: String::from(user_id),
        };

        self.users.push(user.clone());
        match self
            .notifications_tx
            .send(UserNotification::NewUser(user))
        {
            Ok(()) => Ok(()),
            Err(_) => Err("could not send add_user notification"),
        }
    }

    pub fn remove_user(&mut self, user_id: &str) -> Result<(), String> {
        println!("remove user {}", user_id);

        let user = match self.users.iter().position(|v| v.id == user_id) {
            Some(index) => self.users.remove(index),
            None => return Err(String::from("user id not found")),
        };

        match self
            .notifications_tx
            .send(UserNotification::UserRemoved(user))
        {
            Ok(()) => Ok(()),
            Err(_) => Err(String::from("could not send remove_user notification")),
        }
    }
}
