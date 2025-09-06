use std::collections::HashMap;
use reqwest::cookie::CookieStore;
use std::sync::{RwLock, OnceLock};
use std::sync::atomic::{AtomicPtr, Ordering};

/// Unique identifier type for chat sessions (e.g., `Telegram chat_id`).
pub type ChatId = String;

/// نگهداری سشن های مربوط به سامانه ترب
#[derive(Debug, Clone)]
pub struct SessionState {
    pub client: reqwest::Client,
    pub jar: std::sync::Arc<reqwest::cookie::Jar>,
    pub base: reqwest::Url,
}

/// HashMap to store sessions by `chat_id`
static SESSION_MAP: OnceLock<RwLock<HashMap<ChatId, SessionState>>> = OnceLock::new();

/// دسترسی ساده در بقیهٔ کد:
fn get_session_store() -> &'static RwLock<HashMap<ChatId, SessionState>> {
    SESSION_MAP.get_or_init(|| RwLock::new(HashMap::new()))
}

/// For compatibility
pub static SESSION: once_cell::sync::OnceCell<SessionState> = once_cell::sync::OnceCell::new();

/// پوینتر اتمیک برای override (چندبار نوشتن)
static SESSION_OVERRIDE: OnceLock<AtomicPtr<SessionState>> = OnceLock::new();

fn override_ptr() -> &'static AtomicPtr<SessionState> {
    SESSION_OVERRIDE.get_or_init(|| AtomicPtr::new(std::ptr::null_mut()))
}

/// Returns the session for a specific chat_id.
pub fn session_by_chat(chat_id: ChatId) -> Option<SessionState> {
    let store = get_session_store().read().expect("SESSION_MAP lock poisoned");
    store.get(&chat_id).cloned()
}

/// Sets a session for a specific `chat_id`.
pub fn set_session_by_chat(chat_id: ChatId, state: SessionState) {
    let mut store = get_session_store().write().expect("SESSION_MAP lock poisoned");
    store.insert(chat_id, state);
}

/// Removes the session for a given `chat_id`.
pub fn remove_session_by_chat(chat_id: ChatId) -> Option<SessionState> {
    let mut store = get_session_store().write().expect("SESSION_MAP lock poisoned");
    store.remove(&chat_id)
}

/// Backward compatibility to retrieve the default session.
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

/// Optionally retrieves the current CSRF token from a session.
///
/// This function is backward-compatible and assumes the default session
/// if `chat_id`-specific behavior is not needed.
pub fn current_csrftoken(chat_id: Option<ChatId>) -> Option<String> {
    let s = chat_id
        .and_then(session_by_chat) // Prioritize chat_id session if provided (owned SessionState)
        .or_else(|| session().cloned()) // Fallback to the default session (convert borrowed to owned)
        ?;

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