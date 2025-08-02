use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TransientContact {
    pub email: Arc<String>,
    pub display_name: Arc<String>,
    pub presence: Option<Arc<String>>,
    pub msn_object: Option<Arc<String>>,
    pub in_forward_list: bool,
    pub in_allow_list: bool,
    pub in_block_list: bool,
}
