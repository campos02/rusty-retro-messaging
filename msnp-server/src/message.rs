use crate::switchboard::session::Session;
use tokio::sync::broadcast::Sender;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Message {
    Get(String),

    Set {
        key: String,
        value: Sender<Message>,
    },

    Remove(String),

    Value {
        key: String,
        value: Option<Sender<Message>>,
    },

    ToContact {
        sender: String,
        message: String,
        disconnecting: bool,
    },

    GetSession(String),

    SetSession {
        key: String,
        value: Session,
    },

    RemoveSession(String),

    Session {
        key: String,
        value: Option<Session>,
    },

    ToPrincipals {
        sender: String,
        message: String,
        disconnecting: bool,
    },
}
