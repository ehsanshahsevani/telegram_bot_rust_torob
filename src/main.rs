use reqwest::cookie::CookieStore;
use reqwest::header::{ACCEPT, CONTENT_TYPE, ORIGIN, REFERER, USER_AGENT};
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::error::Error;
use teloxide::dptree;
use tokio::signal;

use teloxide::prelude::ResponseResult;
use teloxide::{
    dispatching::{
        HandlerExt, UpdateFilterExt,
        dialogue::{Dialogue, InMemStorage},
    },
    prelude::{Bot, Dispatcher, Request, Requester},
    types::{Message, Update},
    utils::command::BotCommands,
};

/// Payload برای آپلود/ثبت تصویر محصول (multipart/form-data)
#[derive(Debug, Clone)]
pub struct ProductImageCreate {
    /// (required) شناسهٔ محصول
    pub pk: u64,

    /// (optional) فایل باینری تصویر؛ یکی از `image` یا `image_url` باید پر شود
    pub image: Option<ImageFile>,

    /// (optional) لینک تصویر؛ اگر مقدار داشته باشد، `image` می‌تواند None باشد
    pub image_url: Option<String>,

    /// (optional) متن جایگزین تصویر (alt)
    pub image_alt: Option<String>,

    /// (optional) آیا این تصویر «پیش‌فرض» باشد؟ (default=false)
    pub default: Option<bool>,
}

/// نگه‌دارندهٔ فایل تصویر برای multipart
#[derive(Debug, Clone)]
pub struct ImageFile {
    /// نام فایل جهت متادیتا در multipart (مثل "photo.jpg")
    pub filename: String,
    /// بایت‌های فایل
    pub bytes: Vec<u8>,
    /// MIME-Type اختیاری (مثل "image/jpeg" یا "image/png")
    pub mime: Option<String>,
}

/// نوع شمارشی برای وضعیت موجودی
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StockType {
    Limited,
    Unlimited,
    Call,
    OutOfStock,
}

/// مدل «ایجاد محصول» برای ارسال به سرویس
/// - فیلدهای اجباری: `name`, `main_category`
/// - سایر فیلدها اختیاری‌اند و اگر `None` باشند، در JSON ارسال نمی‌شوند.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProductCreate {
    // --- Required ---
    /// name of product (required)
    pub name: String,
    /// main category id (required)
    pub main_category: u64,

    /// --- Optional ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub analysis: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub english_name: Option<String>,

    /// other categories id in list
    #[serde(skip_serializing_if = "Option::is_none")]
    pub other_categories: Option<Vec<u64>>,

    /// brand id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand: Option<u64>,

    /// is product digital or not (default: false)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_digital: Option<bool>,

    /// price in tomans
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<u64>,

    /// price before sale in tomans
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compare_at_price: Option<u64>,

    /// is product in special offer (default: false)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub special_offer: Option<bool>,

    /// end date of special offer (e.g. "2025-09-01T23:59:59Z")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub special_offer_end: Option<String>,

    /// dimensions in centimeters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,

    /// weight in grams
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub barcode: Option<String>,

    /// stock type (default: "unlimited")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stock_type: Option<StockType>,

    /// product stock (default: 0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stock: Option<u64>,

    /// maximum quantity in cart (default: 0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_order_quantity: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub guarantee: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_identifier: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_path: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_slug: Option<String>,

    /// product has variants (default: false)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_variants: Option<bool>,

    /// is product available (default: true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available: Option<bool>,

    /// SEO
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seo_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seo_description: Option<String>,

    /// extra fields list
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_fields: Option<Vec<String>>,
}

/// سازنده مربوط به ایجاد محصول با راحتی و سادگی بیشتر
impl ProductCreate {
    /// سازندهٔ کوتاه برای فیلدهای اجباری؛ بقیه فیلدها پیش‌فرض None هستند.
    pub fn new(name: impl Into<String>, main_category: u64) -> Self {
        Self {
            name: name.into(),
            main_category,
            description: None,
            analysis: None,
            english_name: None,
            other_categories: None,
            brand: None,
            is_digital: None,
            price: None,
            compare_at_price: None,
            special_offer: None,
            special_offer_end: None,
            length: None,
            width: None,
            height: None,
            weight: None,
            barcode: None,
            stock_type: None,
            stock: None,
            max_order_quantity: None,
            guarantee: None,
            product_identifier: None,
            old_path: None,
            old_slug: None,
            has_variants: None,
            available: None,
            seo_title: None,
            seo_description: None,
            extra_fields: None,
        }
    }
}

struct LoginModel {
    csrfmiddlewaretoken: String,
    username: String,
    password: String,
}

/// ======= دسته بندی =======
#[derive(Debug, Deserialize, Clone)]
pub struct Category {
    pub id: u64,
    pub name: String,
    pub parent: Option<u64>,
    pub available: bool,
}

#[derive(Debug, Deserialize)]
struct CategoriesPage {
    next: Option<String>,
    previous: Option<String>,
    total_pages: Option<u32>, // گاهی ممکن است نیاید
    current_page: Option<u32>,
    per_page: Option<u32>,
    results: Vec<Value>, // نتایج را بعداً منعطف تبدیل می‌کنیم
}
/// =========================

/// نگهداری سشن های مربوط به سامانه ترب
#[derive(Debug)]
pub struct SessionState {
    pub client: reqwest::Client,
    pub jar: std::sync::Arc<reqwest::cookie::Jar>,
    pub base: reqwest::Url,
}

fn current_csrftoken() -> Option<String> {
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
static SESSION: once_cell::sync::OnceCell<SessionState> = once_cell::sync::OnceCell::new();

/// دسترسی ساده در بقیهٔ کد:
pub fn session() -> Option<&'static SessionState> {
    SESSION.get()
}

/// مدیریت آدرس اصلی سایت برای کاربر
/// ==========================================================
/// ==========================================================
pub static SITE: std::sync::OnceLock<String> = std::sync::OnceLock::new();

/// فقط یک‌بار در شروع برنامه صدا بزنید
pub fn set_site<S: Into<String>>(s: S) {
    SITE.set(s.into()).expect("SITE already set");
}

/// خواندن امن از هر جا
pub fn site() -> &'static str {
    SITE.get().expect("SITE not set").as_str()
}
/// ==========================================================
/// ==========================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // let user_name: String = String::from("admin");
    // let password: String = String::from("CiP6Ds9oby");

    // let csrfmiddlewaretoken: String =
    //     String::from("lz4DjzwH3Q6A6KvPFHRrRQuOQv0GWtrx6jZlqs4CnnwQTIpnxf98JQsyNHf953F8");

    // let origin = "https://np.mixin.website";
    // set_site(origin);

    // let result = login_in_torob(&user_name, &password, site(), &csrfmiddlewaretoken).await;
    // println!("login result = {}", result);

    // (3)
    let bot_token: String = String::from("8358109688:AAE0QOX-s23RxHAr6cpJSWEb76TREPpoF8c");

    // TODO create test product - mock data and send request for save in torob db
    let bot_token_clone: String = bot_token.clone();
    let bot_handle: tokio::task::JoinHandle<()> = tokio::spawn(async move {
        if let Err(e) = create_bot(bot_token_clone).await {
            eprintln!("bot failed: {e}");
        }
    });

    // برنامه را زنده نگه‌دار تا Ctrl+C
    tokio::signal::ctrl_c().await?;
    // در صورت نیاز، تسک بات را جمع کن
    bot_handle.abort();
    let _ = bot_handle.await;

    Ok(())
}

/// روند لاگین در ترب و ذخیره سشن های مربوط به کار با سرویس های ترب
pub async fn login_in_torob(
    user_name: &String,
    password: &String,
    origin: &str,
    csrfmiddlewaretoken: &String,
) -> &'static str {
    use reqwest::header::{ORIGIN, REFERER, USER_AGENT};

    // let origin = "https://np.mixin.website";
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
                if !token.is_empty() {
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
    if !(r.status().is_success() || r.status().is_redirection()) {
        return "http_error";
    }

    let _ = SESSION.set(SessionState { client, jar, base });

    "ok"
}

fn val_to_opt_u64(v: &Value) -> Option<u64> {
    match v {
        Value::Number(n) => n.as_u64(),
        Value::String(s) => s.parse::<u64>().ok(),
        _ => None,
    }
}

fn val_to_bool_default(v: &Value, default: bool) -> bool {
    match v {
        Value::Bool(b) => *b,
        Value::String(s) => match s.as_str() {
            "true" | "1" => true,
            "false" | "0" => false,
            _ => default,
        },
        Value::Number(n) => n.as_u64().map(|x| x != 0).unwrap_or(default),
        _ => default,
    }
}

fn value_to_category(v: &Value) -> Option<Category> {
    let id = v.get("id").and_then(val_to_opt_u64)?;
    let name = v
        .get("name")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let parent = v.get("parent").and_then(val_to_opt_u64);
    let available = v
        .get("available")
        .map(|x| val_to_bool_default(x, false))
        .unwrap_or(false);
    Some(Category {
        id,
        name,
        parent,
        available,
    })
}

/// همهٔ صفحات را می‌خواند و فقط لیست Category برمی‌گرداند؛
pub async fn fetch_categories_from_service(
    start_path_or_url: &str,
) -> Result<Vec<Category>, Box<dyn serde::ser::StdError + Send + Sync + 'static>> {
    // این خط باید خطای Send+Sync بسازد:
    let s = session().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            "no session; call login_in_torob first",
        )
    })?;
    let base = site();

    let mut next_url: String = if start_path_or_url.starts_with("http") {
        start_path_or_url.to_string()
    } else {
        format!("{base}{start_path_or_url}")
    };

    let mut out: Vec<Category> = Vec::new();

    loop {
        let resp: reqwest::Response = s
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

/// ایجاد محصول در سایت ترب، البته بعد از عملیات لاگین
pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>;

pub async fn create_product(
    product: &ProductCreate,
) -> Result<u64, Box<dyn std::error::Error + Send + Sync + 'static>> {
    use reqwest::header::{ACCEPT, CONTENT_TYPE, CONTENT_TYPE as CT, ORIGIN, REFERER, USER_AGENT};

    let s = session().ok_or("no session; call login_in_torob first")?;
    let csrf = current_csrftoken().ok_or("no csrftoken in jar; login first")?;

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

use reqwest::multipart::{Form, Part};

/// آپلود تصویر محصول (فقط فیلد اجباری `image`)
/// برمی‌گرداند: شناسهٔ تصویر (image_id)
pub async fn upload_product_image_file(
    product_id: u64,
    filename: &str,
    image_bytes: Vec<u8>,
) -> Result<u64, Box<dyn std::error::Error + Send + Sync + 'static>> {
    use reqwest::header::{ACCEPT, ORIGIN, REFERER, USER_AGENT};
    use std::io;

    let s = session().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::Other,
            "no session; call login_in_torob first",
        )
    })?;
    let csrf = current_csrftoken()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "no csrftoken in jar; login first"))?;

    // POST /api/management/v1/products/{pk}/images/
    let endpoint = s
        .base
        .join(&format!(
            "/api/management/v1/products/{}/images/",
            product_id
        ))?
        .to_string();
    let referer = s.base.join("/admin/")?.to_string();
    let origin = s.base.as_str().trim_end_matches('/').to_string();

    // حدس ساده MIME از پسوند فایل (اختیاری اما مفید)
    let mime = match filename.to_ascii_lowercase().rsplit('.').next() {
        Some("png") => "image/png",
        Some("webp") => "image/webp",
        Some("gif") => "image/gif",
        _ => "image/jpeg",
    };

    // فقط فیلد اجباری `image`
    let image_part = Part::bytes(image_bytes)
        .file_name(filename.to_owned())
        .mime_str(mime)?;

    let form = Form::new().part("image", image_part);

    let resp = s
        .client
        .post(&endpoint)
        .header(ACCEPT, "application/json")
        .header(REFERER, &referer)
        .header(ORIGIN, &origin)
        .header(USER_AGENT, "reqwest")
        .header("X-CSRFToken", csrf)
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

/// ====== دستورات بات ======
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "دستورات در دسترس:")]
enum Command {
    /// شروع و دیدن راهنما
    #[command(description = "شروع / راهنما")]
    Start,
    /// ساخت محصول جدید
    #[command(description = "ثبت محصول جدید")]
    RegisterAndCreateNewproduct,
    /// انصراف از فرایند جاری
    #[command(description = "انصراف")]
    Cancel,
}

/// ====== مدل وضعیت مکالمه ======
#[derive(Clone, Debug)]
enum State {
    /// استارت اولیه توسط کاربر - نمایش دستورات
    Start,

    ReceiveWebSite,
    ReceiveUserName,
    ReceivePassword {
        user_name: String,
    },

    /// منتظر دریافت نام محصول
    ReceiveProductName,

    /// منتظر دریافت قیمت
    ReceivePrice {
        name: String,
    },

    /// منتظر دریافت شناسه دسته‌بندی
    ReceiveCategoryId {
        name: String,
        price: i64,
    },

    /// منتظر دریافت تصویر محصول
    ReceiveProductImage {
        name: String,
        price: i64,
        category_id: u64,
        category_name: String,
        product_id: u64,
    },
}

impl Default for State {
    fn default() -> Self {
        State::Start
    }
}

/// =========================

/// ====== هندلر دستورات ======

type MyDialogue = Dialogue<State, InMemStorage<State>>;

/// ====== اسکیما و اجرای دیسپچر ======
pub async fn create_bot(token: String) -> HandlerResult {
    let bot: Bot = Bot::new(token);

    bot.get_me().send().await?;

    Dispatcher::builder(
        bot,
        Update::filter_message()
            .enter_dialogue::<Message, InMemStorage<State>, State>()
            .branch(
                dptree::case![State::Start]
                    .branch(dptree::entry().filter_command::<Command>().endpoint(start)),
            )
            .branch(dptree::case![State::ReceiveWebSite].endpoint(receive_website))
            .branch(dptree::case![State::ReceiveUserName].endpoint(receive_user_name))
            .branch(dptree::case![State::ReceivePassword { user_name }].endpoint(receive_password))
            .branch(dptree::case![State::ReceiveProductName].endpoint(receive_name))
            .branch(dptree::case![State::ReceivePrice { name }].endpoint(receive_price))
            .branch(
                dptree::case![State::ReceiveCategoryId { name, price }]
                    .endpoint(receive_category_id),
            )
            .branch(
                dptree::case![State::ReceiveProductImage {
                    name,
                    price,
                    category_id,
                    category_name,
                    product_id
                }]
                .endpoint(receive_product_image),
            ),
    )
    .dependencies(dptree::deps![InMemStorage::<State>::new()])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;

    Ok(())
}

/// کاربر اگر ربات را استارت کند اتفاقات این تابع ران میشود
async fn start(bot: Bot, dialogue: MyDialogue, msg: Message, cmd: Command) -> HandlerResult {
    match cmd {
        Command::Start => {
            bot.send_message(
                msg.chat.id,
                "سلام! برای ثبت محصول جدید /registerandcreatenewproduct را بفرست.\nهر زمان با /cancel انصراف بده.",
            )
            .await?;
            dialogue.update(State::Start).await?;
        }
        Command::RegisterAndCreateNewproduct => {
            bot.send_message(
                msg.chat.id,
                "برای ثبت محصول ابتدا آدرس پنل خود را ارسال کنید",
            )
            .await?;
            dialogue.update(State::ReceiveWebSite).await?;
        }
        Command::Cancel => {
            bot.send_message(msg.chat.id, "روند ایجاد محصول کنسل شد.")
                .await?;
            dialogue.update(State::Start).await?;
        }
    }
    Ok(())
}

// کمک‌تابع: پارس عدد صحیح (اجازه‌ی ویرگول/فاصله می‌دهد)
fn parse_int(s: &str) -> Option<i64> {
    let cleaned: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    if cleaned.is_empty() {
        None
    } else {
        cleaned.parse::<i64>().ok()
    }
}

///دریافت آدرس پنل کاربر
async fn receive_website(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    let Some(text) = msg.text() else {
        bot.send_message(msg.chat.id, "لطفا آدرس وب سایت خود را وارد کنید")
            .await?;
        return Ok(());
    };

    if text.trim().eq_ignore_ascii_case("/cancel") {
        bot.send_message(msg.chat.id, "روند ایجاد محصول کنسل شد.")
            .await?;
        dialogue.update(State::Start).await?;
        return Ok(());
    }

    let website = text.trim().to_string();

    if website.is_empty() {
        bot.send_message(
            msg.chat.id,
            "آدرس خالی است؛ لطفاً دوباره ادرس سایت خود را وارد کنید.",
        )
        .await?;
        return Ok(());
    }

    if website.starts_with("http") && website.contains(".mixin.website") == false {
        bot.send_message(
            msg.chat.id,
            "آدرس سایت باید شامل .mixin.website باشد؛ لطفاً دوباره تلاش کنید.",
        )
        .await?;
        return Ok(());
    }

    set_site(website);

    bot.send_message(msg.chat.id, "لطفا نام کاربری خود را وارد کنید").await?;

    dialogue.update(State::ReceiveUserName).await?;

    Ok(())
}

/// دریافت نام کاربری از کاربر
async fn receive_user_name(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    let Some(text) = msg.text() else {
        bot.send_message(msg.chat.id, "نام کاربری خود را وارد کنید")
            .await?;
        return Ok(());
    };

    if text.trim().eq_ignore_ascii_case("/cancel") {
        bot.send_message(msg.chat.id, "روند ایجاد محصول، کنسل شد.")
            .await?;
        dialogue.update(State::Start).await?;
        return Ok(());
    }

    let user_name = text.trim().to_string();

    if user_name.is_empty() {
        bot.send_message(
            msg.chat.id,
            "نام کاربری اجباری است و نمیتواند خالی باشد.",
        )
            .await?;
        return Ok(());
    }

    bot.send_message(msg.chat.id, "لطفا رمز عبور خود را وارد کنید").await?;

    dialogue.update(State::ReceivePassword { user_name: user_name }).await?;

    Ok(())
}

/// دریافت رمز عبور برای ورود به پنل ادمین ترب
async fn receive_password(bot: Bot, dialogue: MyDialogue, msg: Message, user_name: String) -> HandlerResult {
    let Some(text) = msg.text() else {
        bot.send_message(msg.chat.id, "رمز عبور خود را وارد کنید")
            .await?;
        return Ok(());
    };

    if text.trim().eq_ignore_ascii_case("/cancel") {
        bot.send_message(msg.chat.id, "روند ایجاد محصول، کنسل شد.")
            .await?;
        dialogue.update(State::Start).await?;
        return Ok(());
    }

    let password = text.trim().to_string();

    if password.is_empty() {
        bot.send_message(
            msg.chat.id,
            "رمز عبور اجباری است و نمیتواند خالی باشد.",
        )
            .await?;
        return Ok(());
    }

    let csrfmiddlewaretoken: String =
        String::from("lz4DjzwH3Q6A6KvPFHRrRQuOQv0GWtrx6jZlqs4CnnwQTIpnxf98JQsyNHf953F8");

    let result = login_in_torob(&user_name, &password, site(), &csrfmiddlewaretoken).await;
    
    println!("login result = {} with web: {}, user_name: {} & password: {}", result, site(), user_name, password);

    let result_bool = match result {
        "ok" => true,
        _ => false,
    };

    if result_bool == false {
        bot.send_message(msg.chat.id, "خطا در ورود به سامانه، مجددا تلاش کنید")
            .await?;
        dialogue.update(State::Start).await?;
        return Ok(());
    }

    bot.send_message(msg.chat.id, "نام محصول را وارد کنید").await?;

    dialogue.update(State::ReceiveProductName).await?;

    Ok(())
}

/// دریافت نام محصول در ربات
async fn receive_name(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    let Some(text) = msg.text() else {
        bot.send_message(msg.chat.id, "لطفاً نام محصول را به صورت متن بفرستید.")
            .await?;
        return Ok(());
    };

    if text.trim().eq_ignore_ascii_case("/cancel") {
        bot.send_message(msg.chat.id, "روند ایجاد محصول کنسل شد.")
            .await?;
        dialogue.update(State::Start).await?;
        return Ok(());
    }

    let name = text.trim().to_string();

    if name.is_empty() {
        bot.send_message(
            msg.chat.id,
            "نام خالی است؛ لطفاً دوباره نام محصول را وارد کنید.",
        )
        .await?;
        return Ok(());
    }

    bot.send_message(msg.chat.id, "قیمت محصول را وارد کنید (فقط عدد).")
        .await?;

    dialogue.update(State::ReceivePrice { name }).await?;

    Ok(())
}

async fn receive_price(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    name: String,
) -> HandlerResult {
    let Some(text) = msg.text() else {
        bot.send_message(msg.chat.id, "لطفاً قیمت را به صورت عدد وارد کنید.")
            .await?;
        return Ok(());
    };

    if text.trim().eq_ignore_ascii_case("/cancel") {
        bot.send_message(msg.chat.id, "روند ایجاد محصول کنسل شد.")
            .await?;

        dialogue.update(State::Start).await?;

        return Ok(());
    }

    let Some(price) = parse_int(text) else {
        bot.send_message(
            msg.chat.id,
            "قیمت نامعتبر است؛ فقط عدد وارد کنید (مثلاً 250000).",
        )
        .await?;

        return Ok(());
    };

    // ⬇️ گرفتن و نمایش دسته‌بندی‌ها
    let cats: Vec<Category> =
        fetch_categories_from_service("/api/management/v1/categories/?page=1").await?;

    if cats.is_empty() {
        bot.send_message(msg.chat.id, "هیچ دسته‌ بندی‌ ای یافت نشد.")
            .await?;
        dialogue.update(State::Start).await?;
        return Ok(());
    }

    let txt = categories_to_text(&cats);

    // اگر خیلی بلند بود، تلگرام محدودیت 4096 کاراکتر دارد؛
    // برای سادگی فعلاً یک پیام می‌فرستیم:
    bot.send_message(msg.chat.id, txt).await?;

    bot.send_message(msg.chat.id, "شناسه‌ی دسته‌بندی موردنظر را ارسال کنید.")
        .await?;

    dialogue
        .update(State::ReceiveCategoryId { name, price })
        .await?;

    Ok(())
}

fn parse_u64(s: &str) -> Option<u64> {
    let cleaned: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    if cleaned.is_empty() {
        None
    } else {
        cleaned.parse::<u64>().ok()
    }
}

/// لیست را به متن ساده تبدیل می‌کند
fn categories_to_text(cats: &[Category]) -> String {
    let mut lines: Vec<String> = Vec::new();
    lines.push("لطفاً شناسه‌ی دسته‌بندی را وارد کنید:\n".into());
    for c in cats {
        let parent = c
            .parent
            .map(|p| p.to_string())
            .unwrap_or_else(|| "Root".into());
        let av = if c.available { "✅" } else { "⛔️" };
        lines.push(format!(
            "{}  {:>4} — {}  (parent: {})",
            av, c.id, c.name, parent
        ));
    }
    lines.join("\n")
}

fn to_u64(n: i64) -> Option<u64> {
    if n >= 0 { Some(n as u64) } else { None }
}

/// دریافت شناسه دسته بندی در ربات
async fn receive_category_id(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    name_price: (String, i64),
) -> HandlerResult {
    let (name, price) = name_price;

    let Some(text) = msg.text() else {
        bot.send_message(msg.chat.id, "شناسه‌ی دسته‌بندی را به صورت عدد وارد کنید.")
            .await?;
        return Ok(());
    };

    if text.trim().eq_ignore_ascii_case("/cancel") {
        bot.send_message(msg.chat.id, "روند ایجاد محصول کنسل شد.")
            .await?;
        dialogue.update(State::Start).await?;
        return Ok(());
    }

    let Some(cat_id) = text.trim().parse::<u64>().ok() else {
        bot.send_message(msg.chat.id, "شناسه نامعتبر است؛ فقط عدد بفرستید.")
            .await?;
        return Ok(());
    };

    // گرفتن تازه‌ترین لیست دسته‌بندی‌ها (یا اگر کش داری، از همان استفاده کن)
    let cats: Vec<Category> =
        match fetch_categories_from_service("/api/management/v1/categories/?page=1").await {
            Ok(v) => v,
            Err(e) => {
                bot.send_message(msg.chat.id, format!("خطا در دریافت دسته‌بندی‌ها: {e}"))
                    .await?;
                dialogue.update(State::Start).await?;
                return Ok(());
            }
        };

    let Some(cat) = cats.iter().find(|c| c.id == cat_id) else {
        bot.send_message(
            msg.chat.id,
            "چنین شناسه‌ای در دسته‌بندی‌ها وجود ندارد؛ دوباره تلاش کنید.",
        )
        .await?;
        return Ok(());
    };

    if cat.available == false {
        bot.send_message(
            msg.chat.id,
            "این دسته‌بندی فعال نیست؛ شناسه‌ی دیگری انتخاب کنید.",
        )
        .await?;
        return Ok(());
    }

    // آماده‌سازی مدل ایجاد محصول با کمترین داده
    let price_u64 = match to_u64(price) {
        Some(v) => v,
        None => {
            bot.send_message(msg.chat.id, "قیمت منفی معتبر نیست؛ فرایند متوقف شد.")
                .await?;
            dialogue.update(State::Start).await?;
            return Ok(());
        }
    };

    let mut product = ProductCreate::new(name.clone(), cat.id);
    // اگر نوع فیلدها در ProductCreate، i64 است، همین دو خط را به Some(price) / Some(stock) تغییر بده.
    product.price = Some(price_u64);
    // product.stock = Some(stock_u64);

    // فراخوانی سرویس ایجاد محصول
    let product_id = match create_product(&product).await {
        Ok(id) => id,
        Err(e) => {
            bot.send_message(msg.chat.id, format!("❌ خطا در ایجاد محصول: {e}"))
                .await?;
            dialogue.update(State::Start).await?;
            return Ok(());
        }
    };

    let summary = format!(
        "✅ محصول با موفقیت ایجاد شد.\n\
         ─────────────────────\n\
         نام: {}\n\
         قیمت: {}\n\
         دسته‌بندی: {} (id: {})\n\
         🆔 شناسه محصول: {}\
         \n برای ثبت محصول بعدی روی /start  کلیک کنید",
        name, price, cat.name, cat.id, product_id
    );

    bot.send_message(msg.chat.id, summary).await?;

    // پیام نهایی به کاربر
    let summary = format!("تصویر مربوط به این محصول را آپلود کنید");

    bot.send_message(msg.chat.id, summary).await?;

    dialogue
        .update(State::ReceiveProductImage {
            name,
            price,
            category_id: cat.id,
            category_name: cat.name.clone(),
            product_id,
        })
        .await?;

    Ok(())
}

use reqwest::Client;
use teloxide::net::Download;
use teloxide::payloads::SendPhotoSetters;
use teloxide::types::{InputFile, PhotoSize}; // برای ارسال به سایت مقصد

/// دریافت تصویر پروفایل در ربات
async fn receive_product_image(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    payload: (String, i64, u64, String, u64), // ← تاپل تخت مطابق Available types
) -> HandlerResult {
    let (name, price, category_id, category_name, product_id) = payload;

    // بررسی اینکه آیا پیام حاوی تصویر است
    let Some(photo) = msg.photo() else {
        bot.send_message(msg.chat.id, "لطفاً یک تصویر ارسال کنید.")
            .await?;
        return Ok(());
    };

    let largest_photo = photo.iter().last().unwrap();
    let file_id = largest_photo.file.id.clone();
    let file = bot.get_file(&file_id).await?;
    let file_path = file.path;

    // ارسال تصویر به مقصد (سایت)
    // دانلود بایت‌ها از تلگرام
    let mut bytes: Vec<u8> = Vec::new();
    bot.download_file(&file_path, &mut bytes).await?;

    // یک نام فایل مناسب (از انتهای مسیر تلگرام)
    let filename = file_path.rsplit('/').next().unwrap_or("image.jpg");

    // آپلود به بک‌اند
    upload_product_image_file(product_id, filename, bytes).await?;

    // پیام نهایی به کاربر
    let summary = format!(
        "✅ محصول با موفقیت ایجاد شد.\n\
         ─────────────────────\n\
         نام: {}\n\
         قیمت: {}\n\
         دسته‌بندی: {} (id: {})\n\
         🆔 شناسه محصول: {}\
         \n برای ثبت محصول بعدی روی /start  کلیک کنید",
        name, price, category_name, category_id, product_id
    );

    let caption = summary;

    bot.send_photo(msg.chat.id, InputFile::file_id(file_id.clone()))
        .caption(caption)
        .await?;

    // پایان جریان
    dialogue.update(State::Start).await?;
    Ok(())
}
