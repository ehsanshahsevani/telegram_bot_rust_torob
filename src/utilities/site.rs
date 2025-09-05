use std::sync::{RwLock, OnceLock};

/// مدیریت آدرس اصلی سایت برای کاربر
pub static SITE: OnceLock<RwLock<String>> = OnceLock::new();

/// فقط یک‌بار لاک را آماده می‌کنیم (داخل تابع)
fn get_lock() -> &'static RwLock<String> {
    SITE.get_or_init(|| RwLock::new(String::new()))
}

/// تنظیم مقدار (هر بار قابل تغییر است)
pub fn set_site<S: Into<String>>(s: S) {
    let mut w = get_lock().write().expect("SITE lock poisoned");
    *w = s.into();
}

/// خواندن مقدار از هر جا
pub fn site() -> String {
    let r = get_lock().read().expect("SITE lock poisoned");
    r.clone()
}
