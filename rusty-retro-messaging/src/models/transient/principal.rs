#[derive(Debug, Clone)]
pub struct Principal {
    pub email: String,
    pub display_name: String,
    pub client_id: Option<usize>,
}
