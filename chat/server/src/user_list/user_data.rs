use proto::chat;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Clone)]
pub struct UserData {
    user: chat::User,
    name: String,
    token: String,
    is_online: bool,
    notifications_tx: mpsc::Sender<chat::IncomingNotification>,
}

impl UserData {
    pub fn new(name: String, sender: mpsc::Sender<chat::IncomingNotification>) -> UserData {
        let id = Uuid::new_v4();
        let token = Uuid::new_v4();

        UserData {
            user: chat::User {
                id: id.to_hyphenated().to_string(),
            },
            name: name,
            token: token.to_hyphenated().to_string(),
            is_online: false,
            notifications_tx: sender,
        }
    }

    pub fn user(&self) -> chat::User {
        self.user.clone()
    }

    pub fn id(&self) -> String {
        self.user.id.clone()
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn token(&self) -> String {
        self.token.clone()
    }

    pub fn is_online(&self) -> bool {
        self.is_online
    }

    pub fn set_online(&mut self, is_online: bool) {
        self.is_online = is_online;
    }

    pub fn sender(&self) -> mpsc::Sender<chat::IncomingNotification> {
        self.notifications_tx.clone()
    }
}
