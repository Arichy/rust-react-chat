// @generated automatically by Diesel CLI.

diesel::table! {
    conversations (id) {
        id -> Text,
        room_id -> Text,
        user_id -> Text,
        message -> Text,
        created_at -> Text,
    }
}

diesel::table! {
    rooms (id) {
        id -> Text,
        name -> Text,
        last_message -> Text,
        created_at -> Text,
        owner_id -> Text,
    }
}

diesel::table! {
    rooms_users (room_id, user_id) {
        room_id -> Text,
        user_id -> Text,
    }
}

diesel::table! {
    users (id) {
        id -> Text,
        username -> Text,
        password -> Text,
        created_at -> Text,
    }
}

diesel::joinable!(conversations -> rooms (room_id));
diesel::joinable!(conversations -> users (user_id));
diesel::joinable!(rooms -> users (owner_id));
diesel::joinable!(rooms_users -> rooms (room_id));
diesel::joinable!(rooms_users -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    conversations,
    rooms,
    rooms_users,
    users,
);
