use teloxide::{dptree, Bot};
use teloxide::requests::Request;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::prelude::{Message, Requester, Update};
use crate::telegram_infrastructure::models::state::State;
use crate::telegram_infrastructure::models::command::Command;
use teloxide::dispatching::{Dispatcher, HandlerExt, UpdateFilterExt};

pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>;

pub struct TelegramBot {
    bot: Bot,
}

impl TelegramBot {
    pub fn new(token: String) -> Self {
        Self {
            bot: Bot::new(token),
        }
    }
    
    pub async fn run_dispatcher(&self) -> HandlerResult
    {
        let bot_clone = self.bot.clone();

        bot_clone.get_me().send().await?;

        Dispatcher::builder(
            bot_clone,
            Update::filter_message()
                .enter_dialogue::<Message, InMemStorage<State>, State>()
                .branch(
                    dptree::case![State::Start]
                        .branch(dptree::entry().filter_command::<Command>()
                            .endpoint(crate::telegram_infrastructure::endpoints::start)),
                )
                .branch(dptree::case![State::ReceiveWebSite]
                    .endpoint(crate::telegram_infrastructure::endpoints::receive_website))
                .branch(dptree::case![State::ReceiveToken]
                    .endpoint(crate::telegram_infrastructure::endpoints::receive_token))
                .branch(dptree::case![State::ReceiveProductName]
                    .endpoint(crate::telegram_infrastructure::endpoints::receive_name))
                .branch(dptree::case![State::ReceivePrice { name }]
                    .endpoint(crate::telegram_infrastructure::endpoints::receive_price))
                .branch(dptree::case![State::ReceiveCategoryId { name, price }]
                        .endpoint(crate::telegram_infrastructure::endpoints::receive_category_id),
                )
                .branch(
                    dptree::case![State::ReceiveProductImage {
                    name,
                    price,
                    category_id,
                    category_name,
                    product_id
                }]
                        .endpoint(crate::telegram_infrastructure::endpoints::receive_product_image),
                ),
        )
            .dependencies(dptree::deps![InMemStorage::<State>::new()])
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;

        Ok(())
    }
}