use super::transient_contact::TransientContact;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub email: Arc<String>,
    pub display_name: Arc<String>,
    pub presence: Option<Arc<String>>,
    pub client_id: Option<usize>,
    pub msn_object: Option<Arc<String>>,
    pub personal_message: Option<Arc<String>>,
    pub blp: Arc<String>,
    pub contacts: HashMap<Arc<String>, TransientContact>,
}

impl AuthenticatedUser {
    pub fn new(email: Arc<String>) -> Self {
        AuthenticatedUser {
            email: email.clone(),
            display_name: email,
            presence: None,
            client_id: None,
            msn_object: None,
            personal_message: None,
            blp: Arc::new("AL".to_string()),
            contacts: HashMap::new(),
        }
    }
}
