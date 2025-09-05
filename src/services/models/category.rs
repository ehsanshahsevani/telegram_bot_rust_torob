use serde::Deserialize;
use serde_json::Value;

/// ======= دسته بندی =======
#[derive(Debug, Deserialize, Clone)]
pub struct Category {
    pub id: u64,
    pub name: String,
    pub parent: Option<u64>,
    pub available: bool,
}

#[derive(Debug, Deserialize)]
pub struct CategoriesPage {
    next: Option<String>,
    previous: Option<String>,
    total_pages: Option<u32>, // گاهی ممکن است نیاید
    current_page: Option<u32>,
    per_page: Option<u32>,
    results: Vec<Value>, // نتایج را بعداً منعطف تبدیل می‌کنیم
}