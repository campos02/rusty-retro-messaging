use crate::switchboard::session::Session;
use tokio::sync::broadcast::Sender;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Message {
    GetTx(String),

    SetTx {
        key: String,
        value: Sender<Message>,
    },

    RemoveTx(String),

    Tx {
        key: String,
        value: Option<Sender<Message>>,
    },

    ToContact {
        sender: String,
        receiver: String,
        message: String,
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
    },

    UserCount(u32),
    GetUsers,
    AddUser,
    RemoveUser,
}
