use std::{collections::HashMap, sync::{Arc, LazyLock}};

use parking_lot::RwLock;

static ENV: LazyLock<Arc<RwLock<HashMap<String, String>>>> = LazyLock::new(|| Arc::new(RwLock::new(HashMap::new())));

use std::env::VarError;
use std::ffi::OsStr;

pub fn var<K: AsRef<OsStr>>(key: K) -> Result<String, VarError> {
    let key = key.as_ref().to_string_lossy().to_string();
    ENV.read().get(&key).cloned().ok_or(VarError::NotPresent)
}

pub fn set(key: &str, value: &str) {
    ENV.write().insert(key.to_string(), value.to_string());
}
