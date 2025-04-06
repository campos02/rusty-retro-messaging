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
        messages: String,
        disconnecting: bool,
    },
}
