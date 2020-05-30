#![allow(unused_import_braces, unreachable_pub)]


table! {
    access (id) {
        id -> Binary,
        source_id -> Binary,
        dest_id -> Binary,
        grant_date -> Timestamp,
        granted_by -> Nullable<Binary>,
    }
}

table! {
    access_option (id) {
        id -> Binary,
        access_id -> Binary,
        option -> crate::DbWrapper<core_common::types::AccessOption>,
        value -> Nullable<Text>,
    }
}

table! {
    entity (id) {
        id -> Binary,
        #[sql_name = "type"]
        type_ -> crate::DbWrapper<core_common::types::EntityTypes>,
    }
}

table! {
    event (id) {
        id -> Binary,
        actor_id -> Nullable<Binary>,
        date -> Timestamp,
        details -> Text,
        #[sql_name = "type"]
        type_ -> crate::DbWrapper<core_common::types::EventTypes>,
        object_id -> Nullable<Binary>,
    }
}

table! {
    groups (entity_id) {
        entity_id -> Binary,
        name -> Text,
        system -> Bool,
        oauth_scope -> Nullable<Text>,
        ldap_group -> Nullable<Text>,
    }
}

table! {
    group_admin (group_id, admin_id) {
        group_id -> Binary,
        admin_id -> Binary,
    }
}

table! {
    group_member (group_id, member_id) {
        group_id -> Binary,
        member_id -> Binary,
        add_date -> Timestamp,
        added_by -> Nullable<Binary>,
    }
}

table! {
    public_key (id) {
        id -> Binary,
        entity_id -> Binary,
        #[sql_name = "type"]
        type_ -> Text,
        keydata -> Text,
        comment -> Nullable<Text>,
        keysize -> Nullable<Integer>,
        fingerprint_md5 -> Nullable<Binary>,
        fingerprint_sha256 -> Nullable<Binary>,
        randomart_md5 -> Nullable<Text>,
        randomart_sha256 -> Nullable<Text>,
        upload_date -> Timestamp,
        active -> Bool,
    }
}

table! {
    server (id) {
        id -> Binary,
        hostname -> Text,
        ip_address -> Nullable<Text>,
        name -> Nullable<Text>,
        key_management -> crate::DbWrapper<core_common::types::KeyManagement>,
        authorization -> crate::DbWrapper<core_common::types::AuthorizationType>,
        sync_status -> crate::DbWrapper<core_common::types::SyncStatusType>,
        rsa_key_fingerprint -> Nullable<Text>,
        port -> Integer,
    }
}

table! {
    server_account (entity_id) {
        entity_id -> Binary,
        server_id -> Binary,
        name -> Nullable<Text>,
        sync_status -> crate::DbWrapper<core_common::types::SyncStatusType>,
    }
}

table! {
    server_admin (server_id, entity_id) {
        server_id -> Binary,
        entity_id -> Binary,
    }
}

table! {
    server_note (id) {
        id -> Binary,
        server_id -> Binary,
        entity_id -> Nullable<Binary>,
        date -> Timestamp,
        note -> Text,
    }
}

table! {
    sync_request (id) {
        id -> Binary,
        server_id -> Binary,
        account_id -> Nullable<Binary>,
        processing -> SmallInt,
    }
}

table! {
    users (entity_id) {
        entity_id -> Binary,
        uid -> Text,
        name -> Nullable<Text>,
        email -> Nullable<Text>,
        password -> Nullable<Text>,
        #[sql_name = "type"]
        type_ -> crate::DbWrapper<core_common::types::UserTypes>,
    }
}

allow_tables_to_appear_in_same_query!(
    server,
    server_admin,
    server_account,
    access,
    users,
    groups,
    group_admin,
    entity,
    group_member,
);
