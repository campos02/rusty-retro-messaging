use crate::{message::Message, models::transient::principal::Principal};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub struct Session {
    pub session_tx: broadcast::Sender<Message>,
    pub session_id: Arc<String>,
    pub cki_string: Arc<String>,
    pub principals: Arc<Mutex<HashMap<Arc<String>, Principal>>>,
}
