pub struct Contact {
    pub id: i32,
    pub user_id: i32,
    pub contact_id: i32,
    pub email: String,
    pub guid: String,
    pub display_name: String,
    pub in_forward_list: bool,
    pub in_allow_list: bool,
    pub in_block_list: bool,
}
