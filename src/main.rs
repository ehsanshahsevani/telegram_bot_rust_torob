use telegram_bot_torob::telegram_infrastructure::telegram_bot::TelegramBot;

/// =========================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // let user_name: String = String::from("admin");
    // let password: String = String::from("CiP6Ds9oby");
    //
    // let csrfmiddlewaretoken: String =
    //     String::from("lz4DjzwH3Q6A6KvPFHRrRQuOQv0GWtrx6jZlqs4CnnwQTIpnxf98JQsyNHf953F8");
    //
    // let origin = "https://np.mixin.website";
    // set_site(origin);
    //
    // let result = login_in_torob(&user_name, &password, site(), &csrfmiddlewaretoken).await;
    // println!("login result = {}", result);

    // (3)
    let bot_token: String = String::from("8358109688:AAE0QOX-s23RxHAr6cpJSWEb76TREPpoF8c");

    // TODO create test product - mock data and send request for save in torob db
    let bot_token_clone: String = bot_token.clone();
    let bot_handle: tokio::task::JoinHandle<()> = tokio::spawn(async move {

        let bot = TelegramBot::new(bot_token_clone);

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