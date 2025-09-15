use reqwest::header::{ACCEPT, CONTENT_TYPE, USER_AGENT};
use serde_json::Value;
use crate::services::models::category::Category;

/// همهٔ صفحات را می‌خواند و فقط لیست Category برمی‌گرداند؛
pub async fn fetch_categories_from_service(
    start_path_or_url: &str,
    chat_id: &str,
) -> Result<Vec<Category>, Box<dyn serde::ser::StdError + Send + Sync + 'static>> {
    // این خط باید خطای Send+Sync بسازد:
    let session =
        crate::utilities::session::session_by_chat(chat_id.to_string().clone()).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            "no session; call login first",
        )
    })?;

    let base =
        crate::utilities::site::get_site(chat_id).unwrap();

    let mut next_url: String = if start_path_or_url.starts_with("http") {
        start_path_or_url.to_string()
    } else {
        format!("{base}{start_path_or_url}")
    };

    let mut out: Vec<Category> = Vec::new();

    loop {
        let resp: reqwest::Response = session
            .client
            .get(&next_url)
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, "reqwest")
            .send()
            .await?;

        let ct: String = resp
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_owned())
            .unwrap_or_default();

        let text = resp.text().await?;
        if ct.contains("application/json") == false
            && !text.trim_start().starts_with('{')
            && !text.trim_start().starts_with('[')
        {
            let preview = text.chars().take(400).collect::<String>();
            // به‌جای format!(...).into() از io::Error استفاده کنید:
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "unexpected content-type/body ({}). preview: {}",
                    ct, preview
                ),
            )
                .into());
        }

        let root: Value = serde_json::from_str(&text)?;

        let items_opt: Option<&Vec<Value>> =
            if let Some(arr) = root.get("results").and_then(|v| v.as_array()) {
                Some(arr)
            } else if let Some(arr) = root.get("result").and_then(|v| v.as_array()) {
                Some(arr)
            } else if let Some(arr) = root.as_array() {
                Some(arr)
            } else {
                None
            };

        if let Some(items) = items_opt {
            for it in items {
                if let Some(cat) =
                    crate::services::tools_method::value_to_category(it) {
                    out.push(cat);
                }
            }
        } else {
            let preview = root.to_string();
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "unrecognized JSON shape (no results/result array). preview: {}",
                    preview.chars().take(400).collect::<String>()
                ),
            )
                .into());
        }

        if let Some(nv) = root.get("next").and_then(|x| x.as_str()) {
            if !nv.is_empty() {
                next_url = if nv.starts_with("http") {
                    nv.to_string()
                } else {
                    format!("{base}{nv}")
                };
                continue;
            }
        }
        break;
    }

    Ok(out)
}
