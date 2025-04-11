// @generated automatically by Diesel CLI.

diesel::table! {
    codes (id) {
        id -> Integer,
        code -> Text,
    }
}

diesel::table! {
    contacts (id) {
        id -> Integer,
        user_id -> Integer,
        contact_id -> Integer,
        display_name -> Text,
        in_forward_list -> Bool,
        in_allow_list -> Bool,
        in_block_list -> Bool,
    }
}

diesel::table! {
    group_members (id) {
        id -> Integer,
        group_id -> Integer,
        contact_id -> Integer,
    }
}

diesel::table! {
    groups (id) {
        id -> Integer,
        user_id -> Integer,
        name -> Text,
        guid -> Text,
    }
}

diesel::table! {
    tokens (id) {
        id -> Integer,
        token -> Text,
        valid_until -> Datetime,
        user_id -> Integer,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        email -> Text,
        password -> Text,
        display_name -> Text,
        puid -> Unsigned<Bigint>,
        guid -> Text,
        gtc -> Text,
        blp -> Text,
    }
}

diesel::joinable!(group_members -> contacts (contact_id));
diesel::joinable!(group_members -> groups (group_id));
diesel::joinable!(groups -> users (user_id));
diesel::joinable!(tokens -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    codes,
    contacts,
    group_members,
    groups,
    tokens,
    users,
);
