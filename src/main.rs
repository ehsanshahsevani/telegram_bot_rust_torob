use telegram_bot_torob::telegram_infrastructure::telegram_bot::TelegramBot;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bot_token: String = String::from("8358109688:AAE0QOX-s23RxHAr6cpJSWEb76TREPpoF8c");

    let bot_token_clone: String = bot_token.clone();
    let bot_handle: tokio::task::JoinHandle<()> = tokio::spawn(async move {
        let bot: TelegramBot =
            telegram_bot_torob::telegram_infrastructure::telegram_bot::TelegramBot::new(bot_token_clone);
        if let Err(e) = bot.run_dispatcher().await {
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