use crate::utilities::session::set_session_multi;

/// روند لاگین در ترب و ذخیره سشن های مربوط به کار با سرویس های ترب
pub async fn login_in_torob(
    user_name: &String,
    password: &String,
    origin: &str,
    csrfmiddlewaretoken: &String,
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

    // اگر ست‌کوکی csrftoken داد، از همان استفاده کن؛ وگرنه از ورودی
    let mut csrf_to_send = csrfmiddlewaretoken.clone();
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
            ("csrfmiddlewaretoken", csrf_to_send.as_str()),
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

    set_session_multi(crate::utilities::session::SessionState { client, jar, base });
    
    "ok"
}
