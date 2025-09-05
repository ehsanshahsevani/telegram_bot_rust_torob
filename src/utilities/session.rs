use reqwest::cookie::CookieStore;

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

/// فقط یک‌بار مقداردهی می‌شود و بعداً از همه‌جا قابل دسترسی است
pub static SESSION: once_cell::sync::OnceCell<SessionState> = once_cell::sync::OnceCell::new();

/// دسترسی ساده در بقیهٔ کد:
pub fn session() -> Option<&'static SessionState> {
    SESSION.get()
}