use crate::utilities::site::get_site;
use crate::services::models::product::ProductCreate;

/// Box / Pin / Rc / Arc — فرق‌ها و کاربردها
// Box<T>
//
// چیست؟ مالکیت یکتا روی heap.
//
// هزینه: یک تخصیص heap + عدم جابجایی (تا وقتی خودت move نکنی).
//
// ترد-سیفتی: بستگی به T دارد؛ خودِ Box نه Send است نه Sync مگر T باشد.
//
// کی استفاده کنم؟
//
// وقتی اندازهٔ نوع معلوم نیست/بزرگ است (مثل dyn Trait، recursive enum).
//
// می‌خواهی مالکیت یکتا برگردانی (Box<dyn Write>).
//
// نکته: ساده‌ترین «پوینتر هوشمند»؛ چرخهٔ ارجاع ندارد.
//
// Pin<P>
//
// چیست؟ «قول می‌دهد این مقدار دیگر move نشود». روی یک پوینتر می‌نشیند (Pin<&mut T>, Pin<Box<T>>, …).
//
// هزینه: خودِ Pin هزینه‌ای ندارد؛ محدودیت حرکتی ایجاد می‌کند.
//
// ترد-سیفتی: مثل پوینتر زیربنایی.
//
// کی استفاده کنم؟
//
// برای نوع‌هایی که به آدرس خودشان وابسته‌اند (self-referential)،
//
// یا Future/Generatorهایی که Unpin نیستند: Pin<Box<dyn Future<...>>>.
//
// نکته‌های مهم:
//
// اگر T: Unpin باشد، Pin تقریباً بی‌اثر است.
//
// Pin<Arc<T>>/Pin<Rc<T>> معمولاً بی‌فایده است چون دسترسی &mut T ندارید؛ pin بیشتر با Pin<Box<T>> یا Pin<&mut T> معنی می‌دهد.
//
// Rc<T>
//
// چیست؟ شمارندهٔ ارجاع غیراتمی (اشتراک در یک ترد).
//
// هزینه: شمارش مرجع ارزان (غیراتمی).
//
// ترد-سیفتی: Thread-unsafe؛ Rc<T> را跨-thread نبرید.
//
// کی استفاده کنم؟
//
// اشتراک یک داده بین چند مالک داخل یک ترد (GUI، بازی، ساخت AST).
//
// نکته: برای تغییر همزمان معمولاً با RefCell<T> می‌آید: Rc<RefCell<T>>.
// مراقب چرخهٔ مرجع باشید؛ نشت حافظه می‌دهد.
//
// Arc<T>
//
// چیست؟ شمارندهٔ ارجاع اتمی (اشتراک بین چند ترد).
//
// هزینه: افزایش/کاهش شمارنده با عملیات اتمی ⇒ کندتر از Rc.
//
// ترد-سیفتی: اگر T: Send + Sync باشد، Arc<T> را می‌توانید بین تردها به‌اشتراک بگذارید.
//
// کی استفاده کنم؟
//
// دادهٔ مشترک خواندنی یا با همگام‌سازی در چند ترد: Arc<T>, Arc<Mutex<T>>, Arc<RwLock<T>>.
//
// نکته: مثل Rc می‌تواند چرخه بسازد (با Weak مدیریت کنید).
//
// جمع‌بندی سریع (Cheat-sheet)
//
// مالکیت یکتا، ساده، heap: Box<T>
//
// حرکت‌ناپذیر کردن شیء (self-ref/Future): Pin<Box<T>> یا Pin<&mut T>
//
// اشتراک در یک ترد: Rc<T> (برای تغییر: Rc<RefCell<T>>)
//
// اشتراک بین تردها: Arc<T> (برای تغییر: Arc<Mutex<T>> یا Arc<RwLock<T>>)
//
// Trait object لازم دارد پوینتر: Box<dyn Trait>, Rc<dyn Trait>, Arc<dyn Trait + Send + Sync>

pub async fn create_product(
    product: &crate::services::models::product::ProductCreate,
    chat_id: String,
) -> Result<u64, Box<dyn std::error::Error + Send + Sync + 'static>> {

    use reqwest::header::{ACCEPT, CONTENT_TYPE, CONTENT_TYPE as CT, ORIGIN, REFERER, USER_AGENT};

    let site = get_site(&chat_id).unwrap();

    let endpoint = format!("{}/api/management/v1/products/", site);
    let referer = format!("{}/admin/",site);
    let origin = site.trim_end_matches('/').to_string();

    let price_string =
        product.price.expect("REASON").to_string();

    let token =
        crate::utilities::token::get_token(chat_id).expect("no token");

    if token.is_empty() {
        return Err("no token".into());
    }
    
    // فرم مولتی‌پارت طبق اسکیما:
    let form= reqwest::multipart::Form::new()
        .text("name", product.name.to_string())
        .text("price", price_string)
        .text("main_category", product.main_category.to_string());

    let client = reqwest::Client::new();

    let resp = client
        .post(endpoint)
        // پاسخ JSON می‌خواهیم
        .header(ACCEPT, "application/json")
        .header(USER_AGENT, "reqwest")
        .header(REFERER, &referer)
        .header(ORIGIN, &origin)
        // ApiKeyAuth در هدر Authorization
        .header(reqwest::header::AUTHORIZATION, format!("Api-Key {}", token))
        // بدنه مولتی‌پارت
        .multipart(form)
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

pub async fn create_product_with_custom_auth()
    -> Result<u64, Box<dyn std::error::Error + Send + Sync + 'static>> {
    use reqwest::header::{ACCEPT, CONTENT_TYPE, CONTENT_TYPE as CT, ORIGIN, REFERER, USER_AGENT};

    let product: ProductCreate = ProductCreate::new("کیف", 1);

    let token = "amOVvXAM6yzeqqJeFb1dWdEUn9cmBs19rAEzP-a3bZIHZiOviL-2y4pTJU2d08bF";

    let endpoint = String::from("https://np.mixin.website/api/management/v1/products/");
    let referer = String::from("https://np.mixin.website/admin/");
    let origin = String::from("https://np.mixin.website");

    let http_client = reqwest::Client::new();

    // فرم مولتی‌پارت طبق اسکیما:
    let form = reqwest::multipart::Form::new()
        .text("name", product.name.to_string())
        .text("main_category", product.main_category.to_string());

    let client = reqwest::Client::new();
    let resp = client
        .post(endpoint)
        // پاسخ JSON می‌خواهیم
        .header(ACCEPT, "application/json")
        .header(USER_AGENT, "reqwest")
        .header(REFERER, &referer)
        .header(ORIGIN, &origin)
        // ApiKeyAuth در هدر Authorization
        .header(reqwest::header::AUTHORIZATION, format!("Api-Key {}", token))
        // بدنه مولتی‌پارت
        .multipart(form)
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

#[cfg(test)]
mod test_create_product {
    use std::error::Error;
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_create_product_success() {
        let result: Result<u64, Box<dyn Error + Send + Sync>> = create_product_with_custom_auth().await;
        assert!(result.is_ok());
    }
}
