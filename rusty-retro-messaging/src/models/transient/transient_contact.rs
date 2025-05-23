#[derive(Debug, Clone)]
pub struct TransientContact {
    pub email: String,
    pub display_name: String,
    pub presence: Option<String>,
    pub msn_object: Option<String>,
    pub in_forward_list: bool,
    pub in_allow_list: bool,
    pub in_block_list: bool,
}
