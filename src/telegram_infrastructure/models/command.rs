/// ====== دستورات بات ======
#[derive(teloxide::macros::BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "دستورات در دسترس:")]
pub enum Command {
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