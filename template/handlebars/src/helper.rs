#![allow(clippy::integer_arithmetic)]
use core_common::serde_json::Value;
use handlebars::{handlebars_helper, HelperDef};

pub(crate) fn get_helpers() -> Vec<(&'static str, Box<dyn HelperDef>)> {
    vec![
        ("log", Box::new(log)),
        ("add", Box::new(add)),
        ("sub", Box::new(sub)),
        ("gt", Box::new(gt)),
        ("lt", Box::new(lt)),
        ("and", Box::new(and)),
        ("contains", Box::new(contains)),
        ("plural", Box::new(plural)),
        ("is_admin", Box::new(is_admin)),
        ("transform_sync", Box::new(transform_sync)),
        ("transform_sync_label", Box::new(transform_sync_label)),
        ("transform_config", Box::new(transform_config)),
    ]
}

handlebars_helper!(log: |v: Json| {
    format!("{:?}", v)
});

handlebars_helper!(add: |v1: u64, v2: u64| {
    v1 + v2
});

handlebars_helper!(sub: |v1: u64, v2: u64| {
    v1 - v2
});

handlebars_helper!(gt: |v1: u64, v2: u64| {
    v1 > v2
});

handlebars_helper!(lt: |v1: u64, v2: u64| {
    v1 < v2
});

handlebars_helper!(and: |v1: bool, v2: bool| {
    v1 && v2
});

handlebars_helper!(contains: |v1: Json, v2: Json| {
    if let (Value::String(v1), Value::String(v2)) = (v1, v2) {
        v1.contains(v2)
    } else {
        match v1 {
            Value::Null | Value::Object(_) => false,
            Value::Bool(_) | Value::Number(_) | Value::String(_) => v1 == v2,
            Value::Array(v1) => v1.contains(v2),
        }
    }
});

handlebars_helper!(plural: |len: u64| {
    if len == 1 {
        ""
    } else {
        "s"
    }
});

handlebars_helper!(is_admin: |type_: str| {
    type_ == "Admin"
});

handlebars_helper!(transform_sync_label: |sync_status: str, key_mgmnt: str| {
    if key_mgmnt == "Keys" {
        match sync_status {
            "NotSyncedYet" | "SyncWarning" => "warning",
            "SyncFailure" => "danger",
            "SyncSuccess" => "success",
            _ => ""
        }
    } else {
        ""
    }
});

handlebars_helper!(transform_sync: |sync_status: str| {
    match sync_status {
        "NotSyncedYet" => "Not synced yet",
        "SyncWarning" => "Sync Warning",
        "SyncFailure" => "Sync Failure",
        "SyncSuccess" => "Sync Success",
        _ => ""
    }
});

handlebars_helper!(transform_config: |key_mgmnt: str, auth: str| {
    match key_mgmnt {
        "Keys" => match auth {
            "Manual" => "Manual account management",
            "Automatic" => "Automatic account management",
            _ => "",
        },
        "Other" => "Managed by another system",
        "None" => "Unmanaged",
        _ => "",
    }
});
