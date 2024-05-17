use std::any;

use dotenv::dotenv;
use minotari_node_grpc_client::grpc::FetchMatchingUtxosRequest;
use minotari_node_grpc_client::grpc::SearchUtxosRequest;
use minotari_node_grpc_client::BaseNodeGrpcClient;
use rusqlite::params;
use rusqlite::Connection;
use tari_crypto::commitment;
use tari_crypto::ristretto::{RistrettoComSig, RistrettoPublicKey, RistrettoSecretKey};
use tari_utilities::hex::Hex;
use tari_utilities::ByteArray;
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
use tari_crypto::commitment::HomomorphicCommitment;

async fn check_commitment_exists(commitment_and_signature: &str) -> Result<bool, anyhow::Error> {
    let mut client = BaseNodeGrpcClient::connect("http://127.0.0.1:18182").await?;
    let regex = regex::Regex::new(r"^[0-9a-fA-F]{64}$").unwrap();

    let commitment = regex
        .find(commitment_and_signature)
        .ok_or_else(|| anyhow!("No commitment found"))?
        .as_str();
    dbg!(commitment);

    // let split = commitment_and_signature.split("::");
    // let collect: Vec<&str> = split.collect();
    // if collect.len() != 4 {
    //     return Err(anyhow!("Invalid commitment and signature"));
    // }
    // let commitment = collect[0];
    // let signature_nonce =
    //     Vec::<u8>::from_hex(collect[1]).map_err(|_| anyhow!("bad hex signature_nonce"))?;
    // let signature_u =
    //     Vec::<u8>::from_hex(collect[2]).map_err(|_| anyhow!("bad hex signature_u"))?;
    // let signature_v =
    //     Vec::<u8>::from_hex(collect[3]).map_err(|_| anyhow!("bad hex signature_v"))?;

    let commitment_bytes =
        Vec::<u8>::from_hex(commitment).map_err(|e| anyhow!("bad hex commitment"))?;

    dbg!(&commitment_bytes);
    // let pub_key_bytes = RistrettoPublicKey::from_canonical_bytes(&commitment_bytes)
    //     .map_err(|_| anyhow!("commitment is not a pub key"))?;
    // let comm_sig = RistrettoComSig::new(
    //     HomomorphicCommitment::from_public_key(
    //         &RistrettoPublicKey::from_canonical_bytes(&signature_nonce)
    //             .map_err(|_| anyhow!("Signature nonce is not a pub key"))?,
    //     ),
    //     RistrettoSecretKey::from_canonical_bytes(&signature_u)
    //         .map_err(|_| anyhow!("signature_u is not a valid key"))?,
    //     RistrettoSecretKey::from_canonical_bytes(&signature_v)
    //         .map_err(|_| anyhow!("signature_v is not a valid key"))?,
    // );

    let mut utxos = client
        .fetch_matching_utxos(FetchMatchingUtxosRequest {
            hashes: vec![commitment_bytes.clone()],
            include_spent: true,
            include_burnt: true,
        })
        .await?
        .into_inner();

    while let Some(utxo) = utxos.next().await {
        dbg!("found by hash");
        let inner = utxo.unwrap();
        let features = inner.output.unwrap().features;
        if features.unwrap().output_type == 2 {
            return Ok(true);
        }
        // dbg!(inner.output.unwrap().features);

        // return Ok(true);
        // dbg!(utxo);
    }

    let mut res = client
        .search_utxos(SearchUtxosRequest {
            commitments: vec![commitment_bytes],
        })
        .await?
        .into_inner();

    let mut count_utxos = 0;
    while let Some(utxo) = res.next().await {
        dbg!(utxo);
        count_utxos += 1;
    }
    Ok(count_utxos > 0)
}

fn ensure_db() -> Result<(), anyhow::Error> {
    let conn = Connection::open("users.db")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
                  handle TEXT NOT NULL,
                  tari_proof TEXT
                  )",
        params![],
    )?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // teloxide::enable_logging!();
    log::info!("Starting bot...");
    dotenv().ok();

    ensure_db()?;

    let bot = Bot::from_env();

    teloxide::repl(bot, |bot: Bot, message: Message| async move {
        if let Some(new_members) = message.new_chat_members() {
            // handle_new_members(message, new_members).await
            for new_member in new_members {
                // bot.send_dice(message.chat.id).await?;
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

            // bot.send_dice(message.chat.id).await?;
            Ok(())
        }
    })
    .await;

    Ok(())
}
