/*
 * Copyright (c) 2025 Matteo Franceschini
 * All rights reserved.
 *
 * Use of this source code is governed by BSD-3-Clause-Clear
 * license that can be found in the LICENSE file
 */
use reqwest::{Client, StatusCode, Url};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod users;

#[derive(Serialize)]
pub struct BookmarkCreateRequest {
    labels: Vec<String>,
    url: Url,
}

impl BookmarkCreateRequest {
    pub fn new(url: Url, bot_mark: bool) -> BookmarkCreateRequest {
        let labels = if bot_mark {
            vec!["readeck-bot".to_owned()]
        } else {
            vec![]
        };
        BookmarkCreateRequest { labels, url }
    }
}

#[derive(Deserialize)]
pub struct BookmarkDetailsResponse {
    pub title: String,
    pub reading_time: Option<u32>,
}

#[derive(Debug, Error)]
pub enum ReadeckError {
    #[error("The request token found in the Authorization header is not valid")]
    Unauthorized,
    #[error(
        "The user doesn't have permission to fetch users for the specified, but has other account permissions"
    )]
    Forbidden,
    #[error("Input data is not valid")]
    InvalidData,
    #[error("HTTP Error {0}")]
    OtherHttp(u16),
    #[error("A network error occurred: {0}")]
    ReqwestError(reqwest::Error),
    #[error("A serialization/deserialization error occurred: {0}")]
    SerdeError(serde_json::Error),
    #[error("Missing bookmark id")]
    MissingBookmarkId,
}

pub struct ReadeckApi {
    client: Client,
    server_url: Url,
}

impl ReadeckApi {
    pub fn new(server_url: Url) -> ReadeckApi {
        let client = reqwest::Client::new();
        ReadeckApi { client, server_url }
    }
    pub async fn save_url(
        &self,
        url: Url,
        api_token: &str,
        bot_mark: bool,
    ) -> Result<String, ReadeckError> {
        let endpoint = self
            .server_url
            .join("/api/bookmarks")
            .expect("Malformed server url");
        let body = BookmarkCreateRequest::new(url, bot_mark);
        let body = serde_json::to_string(&body).map_err(ReadeckError::SerdeError)?;
        let response = self
            .client
            .post(endpoint)
            .bearer_auth(api_token)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .map_err(ReadeckError::ReqwestError)?;
        match response.status() {
            StatusCode::ACCEPTED => {
                let bookmark_id = response
                    .headers()
                    .get("Bookmark-Id")
                    .ok_or(ReadeckError::MissingBookmarkId)?
                    .to_str()
                    .map_err(|_| ReadeckError::MissingBookmarkId)?;
                Ok(bookmark_id.to_owned())
            }
            StatusCode::UNAUTHORIZED => Err(ReadeckError::Unauthorized),
            StatusCode::FORBIDDEN => Err(ReadeckError::Forbidden),
            StatusCode::UNPROCESSABLE_ENTITY => Err(ReadeckError::InvalidData),
            status_code => Err(ReadeckError::OtherHttp(status_code.as_u16())),
        }
    }
    pub async fn get_bookmark_details(
        &self,
        id: String,
        api_token: &str,
    ) -> Result<BookmarkDetailsResponse, ReadeckError> {
        let path = format!("/api/bookmarks/{id}");
        let endpoint = self.server_url.join(&path).expect("Malformed server url");
        let response = self
            .client
            .get(endpoint)
            .bearer_auth(api_token)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(ReadeckError::ReqwestError)?;
        match response.status() {
            StatusCode::OK => {
                let response = response.text().await.map_err(ReadeckError::ReqwestError)?;
                let bookmark_details: BookmarkDetailsResponse =
                    serde_json::from_str(&response).map_err(ReadeckError::SerdeError)?;
                Ok(bookmark_details)
            }
            StatusCode::UNAUTHORIZED => Err(ReadeckError::Unauthorized),
            StatusCode::FORBIDDEN => Err(ReadeckError::Forbidden),
            status_code => Err(ReadeckError::OtherHttp(status_code.as_u16())),
        }
    }
}
