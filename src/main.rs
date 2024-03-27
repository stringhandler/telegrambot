use minotari_node_grpc_client::grpc::SearchUtxosRequest;
use minotari_node_grpc_client::BaseNodeGrpcClient;
use tari_utilities::hex::Hex;
use teloxide::types::{Message, User};
use teloxide::{prelude::*, RequestError};
use tokio_stream::StreamExt;
// #[derive(BotCommand)]
// #[command(
//     rename = "lowercase",
//     description = "These commands are understood by the bot:"
// )]
// enum Command {
//     #[command(description = "Shows a help message listing all commands")]
//     Help,
// }

// async fn handle_new_members(
//     cx: UpdateWithCx<Message>,
//     new_members: Vec<User>,
// ) -> ResponseResult<()> {
//     for new_member in new_members {
//         cx.reply_to(format!(
//             "Welcome, {}! Please send a message to confirm your intent to join.",
//             new_member.first_name
//         ))
//         .send()
//         .await?;

//         let new_member_id = new_member.id;
//         let chat_id = cx.chat_id();

//         let bot = cx.bot.clone();
//         let handler = cx.update_handler();

//         tokio::spawn(async move {
//             let mut stream = handler.stream();

//             while let Some(update) = stream.next().await {
//                 if let Ok(Ok(message)) = update {
//                     if let Some(user) = message.from() {
//                         if user.id == new_member_id {
//                             if let Err(err) = handle_confirmation(&bot, chat_id, message).await {
//                                 log::error!("Error handling confirmation: {}", err);
//                             }
//                         }
//                     }
//                 }
//             }
//         });
//     }

//     Ok(())
// }

// async fn handle_confirmation(
//     bot: &AutoSend<Bot>,
//     chat_id: i64,
//     message: Message,
// ) -> ResponseResult<()> {
//     bot.send_message(
//         chat_id,
//         "Your join request has been approved. Welcome to the group!",
//     )
//     .send()
//     .await?;
//     Ok(())
// }

// async fn handle_commands(rx: DispatcherHandlerRx<UpdateWithCx<Message>>) {
//     teloxide::commands_repl(rx, bot!("YOUR_BOT_TOKEN"), |message, cmd| async move {
//         match cmd {
//             Command::Help => message.reply_to(Command::descriptions()).await?,
//         }
//         Ok(())
//     })
//     .await;
// }
use anyhow::anyhow;

async fn check_commitment_exists(commitment: &str) -> Result<bool, anyhow::Error> {
    let mut client = BaseNodeGrpcClient::connect("http://127.0.0.1:18182").await?;
    let mut res = client
        .search_utxos(SearchUtxosRequest {
            commitments: vec![
                Vec::<u8>::from_hex(commitment).map_err(|e| anyhow!("bad hex commitment"))?
            ],
        })
        .await?
        .into_inner();

    let mut count_utxos = 0;
    while let Some(utxo) = res.next().await {
        //dbg!(utxo);
        count_utxos += 1;
    }
    Ok(count_utxos > 0)
}

#[tokio::main]
async fn main() {
    // teloxide::enable_logging!();
    log::info!("Starting bot...");

    let bot = Bot::from_env();

    teloxide::repl(bot, |bot: Bot, message: Message| async move {
        if let Some(new_members) = message.new_chat_members() {
            // handle_new_members(message, new_members).await
            for new_member in new_members {
                bot.send_dice(message.chat.id).await?;
                // message
                //     .reply_to_message(format!(
                //         "Welcome, {}! Please send a message to confirm your intent to join.",
                //         new_member.first_name
                //     ))
                //     .send()
                //     .await?;
            }
            Ok(())
        } else {
            if let Some(text) = message.text() {
                dbg!("checking commitment");
                let exists = match check_commitment_exists(text).await {
                    Ok(exists) => exists,
                    Err(e) => {
                        bot.send_message(message.chat.id, format!("Error: {}", e))
                            .await?;
                        false
                    }
                };
                if exists {
                    bot.send_message(message.chat.id, "Commitment exists")
                        .await?;
                } else {
                    bot.send_message(message.chat.id, "Commitment does not exist")
                        .await?;
                }
            }

            bot.send_dice(message.chat.id).await?;
            Ok(())
        }
    })
    .await;
}
