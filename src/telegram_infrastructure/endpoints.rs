use crate::services::models::category::Category;
use crate::telegram_infrastructure::models::command::Command;
use crate::telegram_infrastructure::models::state::State;
use crate::telegram_infrastructure::models::state::State::ReceiveProductName;
use crate::utilities::session::remove_session_by_chat;
use crate::utilities::site::{get_site, remove_site};
use crate::utilities::token::{get_token, remove_token, set_token};
use teloxide::Bot;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::net::Download;
use teloxide::payloads::SendPhotoSetters;
use teloxide::prelude::{ChatId, Dialogue, Message};
use teloxide::requests::Requester;
use teloxide::types::InputFile;

type MyDialogue = Dialogue<State, InMemStorage<State>>;
pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>;

/// کاربر اگر ربات را استارت کند اتفاقات این تابع ران میشود
pub async fn start(bot: Bot, dialogue: MyDialogue, msg: Message, cmd: Command) -> HandlerResult {
    match cmd {
        Command::Start => {
            bot.send_message(
                msg.chat.id,
                "سلام! برای ثبت محصول جدید /registerandcreatenewproduct را بفرست.\nبرای حذف اطلاعات قبلی و تغییر توکن از /changetoken استفاده کنید.\nهر زمان با /cancel انصراف بده.",
            )
                .await?;
            dialogue.update(State::Start).await?;
        }
        Command::RegisterAndCreateNewproduct => {
            let mut message = "برای ثبت محصول ابتدا آدرس پنل خود را ارسال کنید";
            let mut start_state = State::ReceiveWebSite;

            let chat_id_telegram = msg.chat.id.0.to_string();

            let site = get_site(&chat_id_telegram);
            let token = get_token(&chat_id_telegram);

            if site.is_some() && token.is_some() {
                message = "نام محصول را وارد کنید";
                start_state = State::ReceiveProductName;
            } else if site.is_some() && token.is_none() {
                message = "توکن خود را وارد کنید";
                start_state = State::ReceiveToken;
            } else {
                remove_token(&chat_id_telegram);
                remove_token(&chat_id_telegram);
            }

            bot.send_message(msg.chat.id, message).await?;
            dialogue.update(start_state).await?;
        }
        Command::Cancel => {
            bot.send_message(msg.chat.id, "روند ایجاد محصول کنسل شد.")
                .await?;
            dialogue.update(State::Start).await?;
        }
        Command::ChangeToken => {
            let chat_id_telegram = msg.chat.id.0.to_string();

            remove_token(&chat_id_telegram);
            remove_site(&chat_id_telegram);

            bot.send_message(
                msg.chat.id,
                "همه اطلاعات شما حذف شد، حالا آدرس پنل خود را وارد کنید",
            )
            .await?;
            dialogue.update(State::ReceiveWebSite).await?;
        }
    }
    Ok(())
}

///دریافت آدرس پنل کاربر
pub async fn receive_website(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
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

    if website.starts_with("https") == false {
        bot.send_message(
            msg.chat.id,
            "آدرس سایت شما نامعتبر میباشد ادرس شما باید با https آغاز شود",
        )
        .await?;

        return Ok(());
    }

    let chat_id_telegram: ChatId = msg.chat.id;

    crate::utilities::site::set_site(
        chat_id_telegram.0.to_string(),
        website.trim_end_matches('/').to_string(),
    );

    bot.send_message(msg.chat.id, "لطفا توکن خود را وارد کنید").await?;
    dialogue.update(State::ReceiveToken).await?;

    Ok(())
}

/// دریافت نام کاربری از کاربر
pub async fn receive_token(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    let Some(text) = msg.text() else {
        bot.send_message(msg.chat.id, "توکن خود را وارد کنید")
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
        bot.send_message(msg.chat.id, "توکن شما اجباری است و نمیتواند خالی باشد.")
            .await?;
        return Ok(());
    }

    let chat_id = msg.chat.id.0.to_string();

    set_token(chat_id, text.trim().to_string());

    bot.send_message(msg.chat.id, "نام محصول را وارد کنید")
        .await?;
    dialogue.update(State::ReceiveProductName).await?;

    Ok(())
}

/// دریافت رمز عبور برای ورود به پنل ادمین ترب
pub async fn receive_password(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    user_name: String,
) -> HandlerResult {
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
        bot.send_message(msg.chat.id, "رمز عبور اجباری است و نمیتواند خالی باشد.")
            .await?;
        return Ok(());
    }

    let chat_id_telegram = msg.chat.id.0.to_string();

    println!("chat_id_telegram = {}", chat_id_telegram);

    let search_value_by_key = crate::utilities::site::get_site(&chat_id_telegram);

    match &search_value_by_key {
        Some(value) => {
            let result = crate::services::accounting::login_in_torob(
                &chat_id_telegram,
                &user_name,
                &password,
                value.as_str(),
            )
            .await;

            println!(
                "login result = {} with web: {}, user_name: {} & password: {}",
                result, value, user_name, password
            );

            let result_bool = match result {
                "ok" => true,
                _ => false,
            };

            if result_bool == false {
                bot.send_message(msg.chat.id, "خطا در ورود به سامانه، لطفا از اول آدرس دقیق سامانه خود و همینطور نام کاربری و رمز عبور خود را مجددا ارسال کنید").await?;
                dialogue.update(State::Start).await?;
                return Ok(());
            }

            bot.send_message(msg.chat.id, "نام محصول را وارد کنید")
                .await?;
            dialogue.update(State::ReceiveProductName).await?;
        }
        None => {
            println!(
                "web site value for this chat id {}, not found.",
                chat_id_telegram
            );
            bot.send_message(
                msg.chat.id,
                "آدرس سایت شما یافت نشد لطفا آدرس سایت خود را وارد کنید",
            )
            .await?;
            dialogue.update(State::Start).await?;
            return Ok(());
        }
    }

    Ok(())
}

/// دریافت نام محصول در ربات
pub async fn receive_name(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
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

    let name: String = text.trim().to_string();

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

pub async fn receive_price(
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

    let chat_id = msg.chat.id.0.to_string();

    // ⬇️ گرفتن و نمایش دسته‌بندی‌ها
    let cats: Vec<Category> =
        match crate::services::category_service::fetch_categories_from_service(&chat_id).await {
            Ok(cats) => cats,
            Err(err) => {
                bot.send_message(msg.chat.id, err.to_string()).await?;
                eprintln!("error in fetching categories: {}", err.to_string());
                return Err(err.into());
            }
        };

    if cats.is_empty() {
        bot.send_message(msg.chat.id, "هیچ دسته‌ بندی‌ ای یافت نشد.")
            .await?;
        dialogue.update(State::Start).await?;
        return Ok(());
    }

    let txt = categories_to_text(&cats);

    // اگر خیلی بلند بود، تلگرام محدودیت 4096 کاراکتر دارد؛
    // برای سادگی فعلاً یک پیام می‌فرستیم:
    // Replace the single message send with this implementation
    let max_length = 4000; // Slightly less than Telegram's 4096 limit
    let chunks = txt
        .chars()
        .collect::<Vec<char>>()
        .chunks(max_length)
        .map(|c| c.iter().collect::<String>())
        .collect::<Vec<String>>();

    for chunk in chunks {
        bot.send_message(msg.chat.id, chunk).await?;
    }
    bot.send_message(msg.chat.id, "شناسه‌ی دسته‌بندی موردنظر را ارسال کنید.")
        .await?;

    dialogue
        .update(State::ReceiveCategoryId { name, price })
        .await?;

    Ok(())
}

// کمک‌تابع: پارس عدد صحیح (اجازه‌ی ویرگول/فاصله می‌دهد)
pub fn parse_int(s: &str) -> Option<i64> {
    let cleaned: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    if cleaned.is_empty() {
        None
    } else {
        cleaned.parse::<i64>().ok()
    }
}

pub fn parse_u64(s: &str) -> Option<u64> {
    let cleaned: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    if cleaned.is_empty() {
        None
    } else {
        cleaned.parse::<u64>().ok()
    }
}

/// لیست را به متن ساده تبدیل می‌کند
pub fn categories_to_text(cats: &[Category]) -> String {
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

pub fn to_u64(n: i64) -> Option<u64> {
    if n >= 0 { Some(n as u64) } else { None }
}

/// دریافت شناسه دسته بندی در ربات
pub async fn receive_category_id(
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

    let chat_id: String = msg.chat.id.0.to_string();

    // گرفتن تازه‌ترین لیست دسته‌بندی‌ها (یا اگر کش داری، از همان استفاده کن)
    let cats: Vec<Category> =
        match crate::services::category_service::fetch_categories_from_service(&chat_id).await {
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

    let mut product = crate::services::models::product::ProductCreate::new(name.clone(), cat.id);
    // اگر نوع فیلدها در ProductCreate، i64 است، همین دو خط را به Some(price) / Some(stock) تغییر بده.
    product.price = Some(price_u64);
    // product.stock = Some(stock_u64);

    // let chat_id = msg.chat.id.0.to_string();

    // فراخوانی سرویس ایجاد محصول
    let product_id = match crate::services::product_service::create_product(&product, chat_id).await
    {
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

/// دریافت تصویر پروفایل در ربات
pub async fn receive_product_image(
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
    let file_path = &file.path;

    let chat_id = msg.chat.id.0.to_string();

    if file.size > 2 * 1024 * 1024 {
        bot.send_message(msg.chat.id, "❌ حجم تصویر باید کمتر از 2 مگابایت باشد.").await?;
        return Ok(());
    }

    // بررسی پسوند مجاز
    let allowed_exts = ["jpg", "jpeg", "png", "gif", "webp"];
    let filename = file_path.rsplit('/').next().unwrap_or("image.jpg");
    let ext_ok = filename
        .rsplit('.')
        .next()
        .map(|ext| allowed_exts.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false);

    if !ext_ok {
        bot.send_message(
            msg.chat.id,
            format!("❌ فرمت فایل پشتیبانی نمی‌شود. فرمت‌های مجاز: {:?}", allowed_exts),
        )
            .await?;
        return Ok(());
    }

    // ارسال تصویر به مقصد (سایت)
    // دانلود بایت‌ها از تلگرام
    let mut bytes: Vec<u8> = Vec::new();
    bot.download_file(&file_path, &mut bytes).await?;

    // یک نام فایل مناسب (از انتهای مسیر تلگرام)
    let filename = file_path.rsplit('/').next().unwrap_or("image.jpg");

    // آپلود به بک‌اند
    crate::services::product_image_service::upload_product_image_file(
        chat_id, product_id, filename, bytes,
    )
    .await?;

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

    let chat_id = msg.chat.id.0.to_string();

    // پایان جریان
    dialogue.update(State::Start).await?;
    Ok(())
}

