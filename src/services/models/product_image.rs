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