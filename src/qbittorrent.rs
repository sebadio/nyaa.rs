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

    #[error("qbittorrent banned this client, too many requests: {0}")]
    Banned(String),
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
    auth_failed: Arc<Mutex<bool>>,
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
            auth_failed: Arc::new(Mutex::new(false)),
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

        let status = resp.status();

        match status {
            StatusCode::FORBIDDEN => {
                // Banned
                let body = resp.text().await.unwrap_or_default();
                Err(Error::Banned(body.trim().to_string()))
            }

            StatusCode::UNAUTHORIZED => {
                let body = resp.text().await.unwrap_or_default();
                Err(Error::Auth(format!("login {status}: {}", body.trim())))
            }

            _ => {
                let body = resp.text().await.unwrap_or_default();
                let body = body.trim();
                match body {
                    "Ok." | "" => Ok(()),
                    other => Err(Error::Auth(format!("login rejected: {other}"))),
                }
            }
        }
    }

    async fn ensure_logged_in(&self) -> Result<(), Error> {
        if *self.auth_failed.lock().unwrap() {
            return Err(Error::Auth(
                "auth previously failed; update credentials to retry".into(),
            ));
        }

        if *self.logged_in.lock().unwrap() {
            return Ok(());
        }

        match self.login().await {
            Ok(()) => {
                *self.logged_in.lock().unwrap() = true;
                Ok(())
            }
            Err(e) => {
                if matches!(e, Error::Banned(_) | Error::Auth(_)) {
                    *self.auth_failed.lock().unwrap() = true;
                }
                Err(e)
            }
        }
    }

    fn invalidate_login(&self) {
        *self.logged_in.lock().unwrap() = false;
    }

    async fn get(&self, path: impl Into<String>) -> Result<reqwest::Response, Error> {
        self.ensure_logged_in().await?;

        let path = path.into();
        log::debug!("Called GET with path: {}", path);

        let url = format!("{}{}", self.base_url, path);

        let resp = self.http.get(&url).send().await?;

        match resp.status() {
            StatusCode::FORBIDDEN | StatusCode::UNAUTHORIZED => {
                self.invalidate_login();
                self.ensure_logged_in().await?;
                let resp = self.http.get(&url).send().await?;

                if resp.status() == StatusCode::FORBIDDEN
                    || resp.status() == StatusCode::UNAUTHORIZED
                {
                    return Err(Error::Auth(format!("forbidden after re-auth: {url}")));
                }
                log::debug!("Re auth worked and response was OK");
                Ok(resp)
            }
            _ => {
                log::debug!("Response was ok");
                Ok(resp)
            }
        }
    }

    pub(crate) fn is_logged_in(&self) -> bool {
        *self.logged_in.lock().unwrap()
    }

    pub(crate) async fn get_torrents(&self) -> Result<Vec<Torrent>, Error> {
        Ok(self
            .get("/api/v2/torrents/info?sort=added_on&reverse=true")
            .await?
            .json::<Vec<Torrent>>()
            .await?)
    }
}
