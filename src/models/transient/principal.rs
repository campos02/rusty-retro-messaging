use std::sync::Arc;

#[derive(Debug)]
pub struct Principal {
    pub email: Arc<String>,
    pub display_name: Arc<String>,
    pub client_id: Option<usize>,
}
