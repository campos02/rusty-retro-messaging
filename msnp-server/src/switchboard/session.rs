use crate::{message::Message, models::transient::principal::Principal};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

#[derive(Clone, Debug)]
pub struct Session {
    pub session_tx: broadcast::Sender<Message>,
    pub session_id: String,
    pub cki_string: String,
    pub principals: Arc<Mutex<Vec<Principal>>>,
}
