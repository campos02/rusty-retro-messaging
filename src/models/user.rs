use std::sync::Arc;

pub struct User {
    pub id: i32,
    pub email: Arc<String>,
    pub password: String,
    pub display_name: String,
    pub puid: u64,
    pub guid: String,
    pub gtc: String,
    pub blp: String,
}
