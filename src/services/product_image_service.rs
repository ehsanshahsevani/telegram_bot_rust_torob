use reqwest::multipart::{Form, Part};

/// آپلود تصویر محصول (فقط فیلد اجباری `image`)
/// برمی‌گرداند: شناسهٔ تصویر (image_id)
pub async fn upload_product_image_file(
    chat_id: String,
    product_id: u64,
    filename: &str,
    image_bytes: Vec<u8>,
) -> Result<u64, Box<dyn std::error::Error + Send + Sync + 'static>> {
    use reqwest::header::{ACCEPT, ORIGIN, REFERER, USER_AGENT};
    use std::io;

    let session =
        crate::utilities::session::session_by_chat(chat_id.clone()).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::Other,
            "no session; call login_in_torob first",
        )
    })?;

    let csrf =
        crate::utilities::session::current_csrftoken(Some(chat_id.to_string()))
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "no csrftoken in jar; login first"))?;

    // POST /api/management/v1/products/{pk}/images/
    let endpoint = session
        .base
        .join(&format!(
            "/api/management/v1/products/{}/images/",
            product_id
        ))?
        .to_string();
    let referer = session.base.join("/admin/")?.to_string();
    let origin = session.base.as_str().trim_end_matches('/').to_string();

    // حدس ساده MIME از پسوند فایل (اختیاری اما مفید)
    let mime = match filename.to_ascii_lowercase().rsplit('.').next() {
        Some("png") => "image/png",
        Some("webp") => "image/webp",
        Some("gif") => "image/gif",
        _ => "image/jpeg",
    };

    // فقط فیلد اجباری `image`
    let image_part =
        Part::bytes(image_bytes)
        .file_name(filename.to_owned())
        .mime_str(mime)?;

    let form = Form::new().part("image", image_part);

    let token =
        crate::utilities::token::get_token(chat_id).expect("no token");

    if token.is_empty() {
        return Err("no token".into());
    }

    let client = reqwest::Client::new();
    
    let resp = client
        .post(&endpoint)
        .header(ACCEPT, "application/json")
        .header(REFERER, &referer)
        .header(ORIGIN, &origin)
        .header(USER_AGENT, "reqwest")
        .header(reqwest::header::AUTHORIZATION, format!("Api-Key {}", token))
        .multipart(form)
        .send()
        .await?;

    let status = resp.status();
    let body = resp.text().await?;

    if !status.is_success() {
        let preview: String = body.chars().take(400).collect();
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("image upload failed: {} • {}", status, preview),
        )
            .into());
    }

    // نمونهٔ پاسخ: { "success": true, "id": 2 }
    #[derive(serde::Deserialize)]
    struct Resp {
        id: Option<u64>,
    }

    if let Ok(r) = serde_json::from_str::<Resp>(&body) {
        if let Some(id) = r.id {
            return Ok(id);
        }
    }
    let v: serde_json::Value = serde_json::from_str(&body)?;
    if let Some(id) = v.get("id").and_then(|x| x.as_u64()) {
        return Ok(id);
    }

    Err(io::Error::new(
        io::ErrorKind::Other,
        format!("image uploaded but could not extract id. body: {}", body),
    )
        .into())
}