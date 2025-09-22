use crate::utilities::session::set_session_by_chat;
// use crate::utilities::session::{set_session_by_chat, set_session_multi, ChatId};
use crate::utilities::token::set_token;

/// روند لاگین در ترب و ذخیره سشن های مربوط به کار با سرویس های ترب
pub async fn login_in_torob(
    chat_id: &str,
    user_name: &String,
    password: &String,
    origin: &str
) -> &'static str {
    use reqwest::header::{ORIGIN, REFERER, USER_AGENT};

    let base = reqwest::Url::parse(&origin).unwrap();
    let url = format!("{}/admin/login/?next=/admin/", &origin);

    let jar = std::sync::Arc::new(reqwest::cookie::Jar::default());
    let client = match reqwest::Client::builder()
        .cookie_provider(jar.clone())
        .build()
    {
        Ok(c) => c,
        Err(_) => return "client_build_error",
    };

    // GET اولیه
    let get_resp = match client.get(&url).header(USER_AGENT, "reqwest").send().await {
        Ok(r) => r,
        Err(_) => return "pre_get_error",
    };
    
    let csrf_to_send;
    
    // اگر ست‌کوکی csrftoken داد، از همان استفاده کن؛ وگرنه از ورودی
    for val in get_resp
        .headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
    {
        if let Ok(s) = val.to_str() {
            if let Some(pos) = s.find("csrftoken=") {
                let rest = &s[pos + "csrftoken=".len()..];
                let token = rest.split(';').next().unwrap_or("").to_string();
                if token.is_empty() == false {
                    csrf_to_send = token;
                    break;
                }
            }
        }
    }

    // POST لاگین
    let resp = client
        .post(&url)
        .header(REFERER, url)
        .header(ORIGIN, origin)
        .header(USER_AGENT, "reqwest")
        .form(&[
            ("username", user_name.as_str()),
            ("password", password.as_str()),
            ("next", "/admin/"),
        ])
        .send()
        .await;

    let r = match resp {
        Ok(r) => r,
        Err(_) => return "request_error",
    };

    if (r.status().is_success() || r.status().is_redirection()) == false {
        return "http_error";
    }

    set_session_by_chat(chat_id.to_string(), crate::utilities::session::SessionState { client, jar, base });
    
    "ok"
}

/// Asynchronously saves a token for a given chat ID.
///
/// # Parameters
/// - `chat_id`: A string slice that represents the unique identifier for the chat.
/// - `token`: A string slice representing the token to be saved for the given chat ID.
///
/// # Behavior
/// This function internally calls the `set_token` function to associate the specified
/// `token` with the provided `chat_id`. Note that this function does not return a result
/// or confirmation, as it assumes the operation is always successful.
///
/// # Examples
/// ```
/// save_token("12345", "example_token").await;
/// ```
///
/// # Notes
/// - The function is asynchronous but does not contain an `await` directly, suggesting
///   that the operation performed within `set_token` might not be asynchronous.
/// - Ensure that `chat_id` and `token` are valid non-empty strings before calling this function.
///
/// # TODO
/// - Verify proper error handling or fallback in case `set_token` fails.
/// - Consider implementing logging or success confirmation for the save operation.
pub async fn save_token(chat_id: &str, token: &str) {
    set_token(chat_id, token);
}
