// use reqwest::cookie::CookieStore;
// 
// /// نگهداری سشن های مربوط به سامانه ترب
// #[derive(Debug)]
// pub struct SessionState {
//     pub client: reqwest::Client,
//     pub jar: std::sync::Arc<reqwest::cookie::Jar>,
//     pub base: reqwest::Url,
// }
// 
// pub fn current_csrftoken() -> Option<String> {
//     let s = session()?;
//     let cookies = s.jar.cookies(&s.base)?;
//     let cookie_str = cookies.to_str().ok()?;
//     cookie_str.split(';').find_map(|p| {
//         let p = p.trim();
//         if let Some(v) = p.strip_prefix("csrftoken=") {
//             Some(v.to_string())
//         } else {
//             None
//         }
//     })
// }
// 
// /// فقط یک‌بار مقداردهی می‌شود و بعداً از همه‌جا قابل دسترسی است
// pub static SESSION: once_cell::sync::OnceCell<SessionState> = once_cell::sync::OnceCell::new();
// 
// /// دسترسی ساده در بقیهٔ کد:
// pub fn session() -> Option<&'static SessionState> {
//     SESSION.get()
// }


use reqwest::cookie::CookieStore;
use std::sync::{OnceLock};
use std::sync::atomic::{AtomicPtr, Ordering};

/// نگهداری سشن های مربوط به سامانه ترب
#[derive(Debug)]
pub struct SessionState {
    pub client: reqwest::Client,
    pub jar: std::sync::Arc<reqwest::cookie::Jar>,
    pub base: reqwest::Url,
}

pub fn current_csrftoken() -> Option<String> {
    let s = session()?;
    let cookies = s.jar.cookies(&s.base)?;
    let cookie_str = cookies.to_str().ok()?;
    cookie_str.split(';').find_map(|p| {
        let p = p.trim();
        if let Some(v) = p.strip_prefix("csrftoken=") {
            Some(v.to_string())
        } else {
            None
        }
    })
}

/// فقط یک‌بار مقداردهی می‌شود و بعداً از همه‌جا قابل دسترسی است (سازگاری با کدهای قبلی)
pub static SESSION: once_cell::sync::OnceCell<SessionState> = once_cell::sync::OnceCell::new();

/// پوینتر اتمیک برای override (چندبار نوشتن)
static SESSION_OVERRIDE: OnceLock<AtomicPtr<SessionState>> = OnceLock::new();

fn override_ptr() -> &'static AtomicPtr<SessionState> {
    SESSION_OVERRIDE.get_or_init(|| AtomicPtr::new(std::ptr::null_mut()))
}

/// دسترسی ساده در بقیهٔ کد:
pub fn session() -> Option<&'static SessionState> {
    // اگر override وجود دارد، همان را برگردان
    let p = override_ptr().load(Ordering::SeqCst);
    if !p.is_null() {
        // ایمن است چون ما Box را leak کرده‌ایم و تا پایان برنامه آزاد نمی‌شود.
        return Some(unsafe { &*p });
    }
    // در غیر این صورت، از همان OnceCell اولیه بخوان
    SESSION.get()
}

/// تابع جدید برای تنظیم/تعویض سشن هر بار که لازم شد.
/// (کدهای قبلی که SESSION.set(...) می‌زنند، همان‌طور کار می‌کنند؛
/// برای آپدیت‌های بعدی از این استفاده کنید.)
pub fn set_session_multi(state: SessionState) {
    // Box::leak تا رفرنس‌های برگشتی از session() همیشه معتبر بمانند.
    let leaked: &'static mut SessionState = Box::leak(Box::new(state));
    // آدرس را در پوینتر اتمیک می‌گذاریم.
    override_ptr().store(leaked as *mut SessionState, Ordering::SeqCst);
    // توجه: عمداً مقدار قبلی را آزاد نمی‌کنیم تا رفرنس‌های قدیمی dangling نشوند.
}
