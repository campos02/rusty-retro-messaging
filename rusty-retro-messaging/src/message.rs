use crate::{
    models::transient::authenticated_user::AuthenticatedUser, switchboard::session::Session,
};
use std::sync::Arc;
use tokio::sync::broadcast::Sender;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Message {
    GetTx(String),

    SetTx {
        key: Arc<String>,
        value: Sender<Message>,
    },

    RemoveTx(Arc<String>),

    Tx {
        key: String,
        value: Option<Sender<Message>>,
    },

    ToContact {
        sender: Arc<String>,
        receiver: Arc<String>,
        message: String,
    },

    GetSession(Arc<String>),

    SetSession {
        key: Arc<String>,
        value: Session,
    },

    RemoveSession(Arc<String>),

    Session {
        key: Arc<String>,
        value: Option<Session>,
    },

    ToPrincipals {
        sender: Arc<String>,
        message: Vec<u8>,
    },

    SendUserDetails {
        sender: Arc<String>,
        receiver: Arc<String>,
        authenticated_user: Option<AuthenticatedUser>,
        protocol_version: Option<u32>,
    },

    UserDetails {
        sender: Arc<String>,
        receiver: Arc<String>,
        authenticated_user: Option<AuthenticatedUser>,
        protocol_version: Option<u32>,
    },

    UserCount(u32),
    GetUsers,
    AddUser,
    RemoveUser,
}
