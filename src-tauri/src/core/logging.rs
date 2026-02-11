use serde_json::Value;

fn normalize_fields(fields: Option<Value>) -> Option<Value> {
    match fields {
        Some(Value::Object(map)) if !map.is_empty() => Some(Value::Object(map)),
        _ => None,
    }
}

fn compose(scope: &str, event: &str, fields: Option<Value>) -> String {
    let prefix = format!("[{}] {}", scope, event);
    match normalize_fields(fields) {
        Some(v) => format!("{} {}", prefix, v),
        None => prefix,
    }
}

pub fn debug(scope: &str, event: &str, fields: Option<Value>) {
    log::debug!("{}", compose(scope, event, fields));
}

pub fn info(scope: &str, event: &str, fields: Option<Value>) {
    log::info!("{}", compose(scope, event, fields));
}

pub fn warn(scope: &str, event: &str, fields: Option<Value>) {
    log::warn!("{}", compose(scope, event, fields));
}

pub fn error(scope: &str, event: &str, fields: Option<Value>) {
    log::error!("{}", compose(scope, event, fields));
}

#[macro_export]
macro_rules! app_debug {
    ($scope:expr, $event:expr) => {
        $crate::core::logging::debug($scope, $event, None)
    };
    ($scope:expr, $event:expr, $fields:expr) => {
        $crate::core::logging::debug($scope, $event, Some($fields))
    };
}

#[macro_export]
macro_rules! app_info {
    ($scope:expr, $event:expr) => {
        $crate::core::logging::info($scope, $event, None)
    };
    ($scope:expr, $event:expr, $fields:expr) => {
        $crate::core::logging::info($scope, $event, Some($fields))
    };
}

#[macro_export]
macro_rules! app_warn {
    ($scope:expr, $event:expr) => {
        $crate::core::logging::warn($scope, $event, None)
    };
    ($scope:expr, $event:expr, $fields:expr) => {
        $crate::core::logging::warn($scope, $event, Some($fields))
    };
}

#[macro_export]
macro_rules! app_error {
    ($scope:expr, $event:expr) => {
        $crate::core::logging::error($scope, $event, None)
    };
    ($scope:expr, $event:expr, $fields:expr) => {
        $crate::core::logging::error($scope, $event, Some($fields))
    };
}
