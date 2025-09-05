use serde::{Deserialize, Serialize};

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