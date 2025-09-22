use crate::services::models::category::Category;
use crate::services::tools_method::value_to_category;
use crate::utilities::site::get_site;
use crate::utilities::token::get_token;
use reqwest::header::{ACCEPT, CONTENT_TYPE, ORIGIN, REFERER, USER_AGENT};
use serde_json::Value;

/// همهٔ صفحات را می‌خواند و فقط لیست Category برمی‌گرداند؛
pub async fn fetch_categories_from_service(
    start_path_or_url: &str,
    chat_id: &str,
) -> Result<Vec<Category>, Box<dyn serde::ser::StdError + Send + Sync + 'static>> {
    let base = get_site(chat_id).unwrap();
    let token = get_token(chat_id).unwrap();

    let mut next_url: String = if start_path_or_url.starts_with("http") {
        start_path_or_url.to_string()
    } else {
        format!("{base}{start_path_or_url}")
    };

    let endpoint = next_url;
    let referer = format!("{}/admin/", base);
    let origin = base.trim_end_matches('/').to_string();

    let mut out: Vec<Category> = Vec::new();

    let http_client = reqwest::Client::new();

    loop {
        let resp: reqwest::Response = http_client
            .get(&endpoint)
            .header(REFERER, &referer)
            .header(ORIGIN, &origin)
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, "reqwest")
            .header(reqwest::header::AUTHORIZATION, format!("Api-Key {}", token))
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
                if let Some(cat) = value_to_category(it) {
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

        let root_next = root.get("next").and_then(|x| x.as_str());

        if let Some(nv) = root_next {
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
