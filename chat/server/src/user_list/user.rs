use super::UserData;
use proto::chat;
use tokio::sync::mpsc;

pub struct User {
    pub user_data: UserData,
    notifications_rx: Option<mpsc::Receiver<chat::IncomingNotification>>,
}

impl User {
    pub fn new(name: &str) -> User {
        let (notifications_tx, notifications_rx) = mpsc::channel(4);

        User {
            user_data: UserData::new(String::from(name), notifications_tx),
            notifications_rx: Some(notifications_rx),
        }
    }

    pub fn take_receiver(&mut self) -> mpsc::Receiver<chat::IncomingNotification> {
        self.notifications_rx.take().unwrap()
    }

    pub fn id(&self) -> String {
        self.user_data.id()
    }
}
