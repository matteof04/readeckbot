/*
 * Copyright (c) 2025 Matteo Franceschini
 * All rights reserved.
 *
 * Use of this source code is governed by BSD-3-Clause-Clear
 * license that can be found in the LICENSE file
 */

use std::{env, process::exit, sync::Arc};

use log::{error, info, trace, warn};
use readeckbot::{
    ReadeckApi, ReadeckError,
    users::{UserData, Users},
};
use regex::Regex;
use reqwest::Url;
use teloxide::{
    prelude::*,
    types::{MessageEntityKind, ReplyParameters},
};
use thiserror::Error;

#[tokio::main]
async fn main() {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(log::LevelFilter::Info)
        .parse_env("LOG_LEVEL")
        .init();
    let pretty_response: bool = env::var("PRETTY_RESPONSE")
        .unwrap_or("true".to_owned())
        .parse()
        .unwrap_or(true);
    let pretty_response = Arc::new(pretty_response);
    let api_url = match env::var("API_URL") {
        Ok(url) => url,
        Err(_) => {
            error!("API_URL not set!");
            exit(1)
        }
    };
    let api_url: Url = match api_url.parse() {
        Ok(url) => url,
        Err(_) => {
            error!("Invalid API URL");
            exit(1)
        }
    };
    let bot_token = match env::var("BOT_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            error!("BOT_TOKEN not set!");
            exit(1)
        }
    };
    let api = ReadeckApi::new(api_url);
    let users = match env::var("USERS_FILE") {
        Ok(f) => f,
        Err(_) => {
            warn!("USERS_FILE not set, default to users.json");
            "users.json".to_owned()
        }
    };
    let users = Users::load(users);
    let users = Arc::new(users);
    let api = Arc::new(api);
    let bot = Bot::new(bot_token);
    let handler = Update::filter_message().endpoint(
        |bot: Bot,
         api: Arc<ReadeckApi>,
         pretty_r: Arc<bool>,
         usr: Arc<Users>,
         msg: Message| async move {
            if let Some(user) = &msg.from {
                match usr.find(user.id.0) {
                    Some(user_data) => {
                        trace!("New message from user with ID: {:?}", user.id);
                        let response = match msg_handler(api, &msg, *pretty_r, user_data).await {
                            Ok(s) => s,
                            Err(e) => format!("{e}"),
                        };
                        bot.send_message(msg.chat.id, response)
                            .reply_parameters(ReplyParameters::new(msg.id))
                            .await?;
                    }
                    None => {
                        info!(
                            "Connection refused with unauthorized user with ID: {:?}",
                            user.id
                        );
                        bot.send_message(msg.chat.id, "Unauthorized")
                            .reply_parameters(ReplyParameters::new(msg.id))
                            .await?;
                    }
                }
            } else {
                bot.send_message(msg.chat.id, "Unauthorized")
                    .reply_parameters(ReplyParameters::new(msg.id))
                    .await?;
            }
            respond(())
        },
    );
    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![api, pretty_response, users])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

#[derive(Debug, Error)]
enum ProcessError {
    #[error("Not a valid URL")]
    NotAnUrl,
    #[error("{0}")]
    ReadeckError(ReadeckError),
    #[error("Article saved, but errors occurred: {0}")]
    SavedWithError(ReadeckError),
}

async fn msg_handler(
    api: Arc<ReadeckApi>,
    msg: &Message,
    pretty_response: bool,
    user_data: &UserData,
) -> Result<String, ProcessError> {
    let url = extract_url(msg).ok_or(ProcessError::NotAnUrl)??;
    let bookmark_id = api
        .save_url(url, &user_data.api_token, user_data.bot_marked)
        .await
        .map_err(ProcessError::ReadeckError)?;
    if pretty_response {
        let bookmark_details = api
            .get_bookmark_details(bookmark_id, &user_data.api_token)
            .await
            .map_err(ProcessError::SavedWithError)?;
        let response = if !bookmark_details.title.is_empty() {
            match bookmark_details.reading_time {
                Some(reading_time) => format!(
                    "{} added to Readeck.\n\n Reading time: {}",
                    bookmark_details.title, reading_time
                ),
                None => format!("{} added to Readeck.", bookmark_details.title),
            }
        } else {
            "Added to Readeck.".to_owned()
        };
        Ok(response)
    } else {
        Ok("Article saved successfully".to_owned())
    }
}

fn extract_url(msg: &Message) -> Option<Result<Url, ProcessError>> {
    let mut urls: Vec<Result<Url, ProcessError>> = vec![];
    if let Some(entities) = msg.parse_entities() {
        let mut parsed_text_links: Vec<Result<Url, ProcessError>> = entities
            .into_iter()
            .filter_map(|e| {
                if let MessageEntityKind::TextLink { url } = e.kind() {
                    Some(Ok(url.to_owned()))
                } else {
                    None
                }
            })
            .collect();
        urls.append(&mut parsed_text_links);
    }
    if let Some(entities) = msg.parse_caption_entities() {
        let mut parsed_caption_links: Vec<Result<Url, ProcessError>> = entities
            .into_iter()
            .filter_map(|e| {
                if let MessageEntityKind::TextLink { url } = e.kind() {
                    Some(Ok(url.to_owned()))
                } else {
                    None
                }
            })
            .collect();
        urls.append(&mut parsed_caption_links);
    }
    let msg_text = msg.text().unwrap_or("");
    let mut parsed_msg_text = parse_url(msg_text);
    urls.append(&mut parsed_msg_text);
    let caption_text = msg.caption().unwrap_or("");
    let mut parsed_caption_text = parse_url(caption_text);
    urls.append(&mut parsed_caption_text);
    urls.into_iter().next()
}

fn parse_url(text: &str) -> Vec<Result<Url, ProcessError>> {
    Regex::new(r"https?:\/\/(www\.)?[-a-zA-Z0-9@:%._\+~#=]{2,256}\.[a-z]{2,4}\b([-a-zA-Z0-9@:%_\+.~#?&//=]*)")
        .unwrap()
        .find_iter(text)
        .map(|m| m.as_str())
        .map(|s| Url::parse(s).map_err(|_| ProcessError::NotAnUrl))
        .collect()
}
