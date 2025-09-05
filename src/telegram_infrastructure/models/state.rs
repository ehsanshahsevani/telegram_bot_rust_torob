/// ====== مدل وضعیت مکالمه ======
#[derive(Clone, Debug)]
pub enum State {
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