use std::collections::HashMap;
use std::sync::{OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub static TOKEN: OnceLock<RwLock<HashMap<String, String>>> = OnceLock::new();

fn get_lock() -> &'static RwLock<HashMap<String, String>> {
    TOKEN.get_or_init(|| RwLock::new(HashMap::new()))
}

/// تنظیم مقدار (هر بار قابل تغییر است)
pub fn set_token<S: Into<String>>(key: S, value: S) {
    let mut w: RwLockWriteGuard<HashMap<String, String>> =
        get_lock().write().expect("SITE lock poisoned");

    // Convert the key once to avoid multiple conversions
    let key_string = key.into();

    if w.contains_key(&key_string) {
        // Only update if the key exists
        w.insert(key_string, value.into());
    }
    else {
        w.insert(key_string, value.into());
    }
}

/// خواندن یک مقدار با استفاده از کلید
pub fn get_token<S: AsRef<str>>(key: S) -> Option<String> {
    let r: RwLockReadGuard<HashMap<String, String>> =
        get_lock().read().expect("TOKEN lock poisoned");

    r.get(key.as_ref()).cloned()
}

/// حذف یک مقدار با کلید
pub fn remove_token<S: AsRef<str>>(key: S) {
    let mut w: RwLockWriteGuard<HashMap<String, String>> =
        get_lock().write().expect("TOKEN lock poisoned");

    w.remove(key.as_ref());
}

/// نمایش همه مقادیر ذخیره‌شده
pub fn list_token() -> Vec<(String, String)> {
    let r: RwLockReadGuard<HashMap<String, String>> =
        get_lock().read().expect("TOKEN lock poisoned");

    r.iter().map(|(key, value)| (key.clone(), value.clone())).collect()
}