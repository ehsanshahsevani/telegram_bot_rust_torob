pub async fn create_product(
    product: &crate::services::models::product::ProductCreate, chat_id: String,
) -> Result<u64, Box<dyn std::error::Error + Send + Sync + 'static>> {
    use reqwest::header::{ACCEPT, CONTENT_TYPE, CONTENT_TYPE as CT, ORIGIN, REFERER, USER_AGENT};

    let s =
        crate::utilities::session::session_by_chat(chat_id.clone())
            .ok_or("no session; call login_in_torob first")?;

    let csrf =
        crate::utilities::session::current_csrftoken(Some(chat_id.to_string()))
            .ok_or("no csrftoken in jar; login first")?;

    let endpoint = s.base.join("/api/management/v1/products/")?.to_string();
    let referer = s.base.join("/admin/")?.to_string();
    let origin = s.base.as_str().trim_end_matches('/').to_string();

    let resp = s
        .client
        .post(&endpoint)
        .header(ACCEPT, "application/json")
        .header(CONTENT_TYPE, "application/json")
        .header(REFERER, &referer)
        .header(ORIGIN, &origin)
        .header(USER_AGENT, "reqwest")
        .header("X-CSRFToken", csrf)
        .json(product)
        .send()
        .await?;

    let status = resp.status();
    let ct_hdr = resp
        .headers()
        .get(CT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let text = resp.text().await?;

    if (status.is_success() || status.is_redirection()) == false {
        let preview: String = text.chars().take(400).collect();
        return Err(format!("product create failed: {} • {}", status, preview).into());
    }
    if ct_hdr.contains("application/json") == false && text.trim_start().starts_with('{') == false {
        return Err(format!("unexpected content-type: {}", ct_hdr).into());
    }

    #[derive(serde::Deserialize)]
    struct Minimal {
        id: u64,
    }
    if let Ok(min) = serde_json::from_str::<Minimal>(&text) {
        return Ok(min.id);
    }
    let v: serde_json::Value = serde_json::from_str(&text)?;
    if let Some(id) = v.get("id").and_then(|x| x.as_u64()) {
        return Ok(id);
    }
    if let Some(id) = v
        .get("result")
        .and_then(|r| r.get("id"))
        .and_then(|x| x.as_u64())
    {
        return Ok(id);
    }

    Err(format!("product created but could not extract id. body: {}", text).into())
}
