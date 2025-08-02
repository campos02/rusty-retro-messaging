use chrono::NaiveDateTime;

pub struct Token {
    pub id: i32,
    pub token: String,
    pub valid_until: NaiveDateTime,
    pub user_id: i32,
}
