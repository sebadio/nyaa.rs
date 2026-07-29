use std::sync::{Arc, Mutex};

use reqwest::{self, StatusCode};
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Clone, Deserialize)]
pub struct Torrent {
    pub name: String,
    pub hash: String,
    pub content_path: String,
    pub size: u64,
    pub progress: f32,
    pub state: String,
    pub num_seeds: u32,
    pub num_leechs: u32,
    pub num_complete: u32,
    pub num_incomplete: u32,
    pub added_on: i64,
}

#[derive(Debug, Error, Clone)]
pub(crate) enum Error {
    #[error("error authenticating against qbittorrent: {0}")]
    Auth(String),

    #[error("qbittorrent request failed: {0}")]
    Request(String),
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Request(e.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct Client {
    http: reqwest::Client,
    base_url: String,
    logged_in: Arc<Mutex<bool>>,
    username: String,
    password: String,
}

impl Client {
    pub(crate) fn new(
        base_url: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<Self, Error> {
        let http = reqwest::Client::builder().cookie_store(true).build()?;

        Ok(Self {
            http,
            base_url: base_url.into(),
            logged_in: Arc::new(Mutex::new(false)),
            username: username.into(),
            password: password.into(),
        })
    }

    async fn login(&self) -> Result<(), Error> {
        let resp = self
            .http
            .post(format!("{}/api/v2/auth/login", self.base_url))
            .header("Referer", &self.base_url)
            .form(&[("username", &self.username), ("password", &self.password)])
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(())
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            Err(Error::Auth(format!("login {status}: {}", body.trim())))
        }
    }

    async fn ensure_logged_in(&self) -> Result<(), Error> {
        if *self.logged_in.lock().unwrap() {
            return Ok(());
        }
        self.login().await?;
        *self.logged_in.lock().unwrap() = true;
        Ok(())
    }

    fn invalidate_login(&self) {
        *self.logged_in.lock().unwrap() = false;
    }

    async fn get(&self, path: impl Into<String>) -> Result<reqwest::Response, Error> {
        self.ensure_logged_in().await?;
        let url = format!("{}{}", self.base_url, path.into());

        let request = self.http.get(&url);
        let resp = request.send().await?;

        match resp.status() {
            StatusCode::FORBIDDEN => {
                self.invalidate_login();
                self.ensure_logged_in().await?;
                Ok(self.http.get(&url).send().await?)
            }
            _ => Ok(resp),
        }
    }

    pub(crate) async fn get_torrents(&self) -> Result<Vec<Torrent>, Error> {
        Ok(self
            .get("/api/v2/torrents/info?sort=added_on&reverse=true")
            .await?
            .json::<Vec<Torrent>>()
            .await?)
    }
}
