use super::transient_contact::TransientContact;
use std::collections::HashMap;

#[derive(Clone)]
pub struct AuthenticatedUser {
    pub email: String,
    pub display_name: String,
    pub presence: Option<String>,
    pub client_id: Option<usize>,
    pub msn_object: Option<String>,
    pub personal_message: Option<String>,
    pub blp: String,
    pub contacts: HashMap<String, TransientContact>,
}

impl AuthenticatedUser {
    pub fn new(email: String) -> Self {
        AuthenticatedUser {
            email: email.clone(),
            display_name: email,
            presence: None,
            client_id: None,
            msn_object: None,
            personal_message: None,
            blp: "AL".to_string(),
            contacts: HashMap::new(),
        }
    }
}
