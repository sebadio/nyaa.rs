use crate::torrent::{Torrent, TorrentPostResponse};
use reqwest::{self, StatusCode, multipart};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum Error {
    #[error("error authenticating against qbittorrent: {0}")]
    Auth(String),

    #[error("qbittorrent request failed: {0}")]
    Request(String),

    #[error("qbittorrent banned this client, too many requests: {0}")]
    Banned(String),

    #[error("torrent disappeard from qBittorrent before finishing: {0}")]
    TorrentNotFound(String),

    #[error("conflict, torrent already exists")]
    AlreadyExists(),
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        match e.status() {
            Some(reqwest::StatusCode::CONFLICT) => Error::AlreadyExists(),
            _ => Error::Request(e.to_string()),
        }
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
    save_path: Option<String>,
}

impl Client {
    pub fn new(
        base_url: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
        save_path: Option<impl Into<String>>,
    ) -> Result<Self, Error> {
        let http = reqwest::Client::builder().cookie_store(true).build()?;

        Ok(Self {
            http,
            base_url: base_url.into(),
            logged_in: Arc::new(Mutex::new(false)),
            auth_failed: Arc::new(Mutex::new(false)),
            username: username.into(),
            password: password.into(),
            save_path: save_path.map(Into::into),
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

    async fn get<T: Serialize + ?Sized>(
        &self,
        path: impl Into<String>,
        query: Option<&T>,
    ) -> Result<reqwest::Response, Error> {
        self.ensure_logged_in().await?;

        let path = path.into();
        let url = format!("{}{}", self.base_url, path);
        let req_builder = self.http.get(&url);

        let req_builder = if let Some(q) = query {
            req_builder.query(q)
        } else {
            req_builder
        };

        let resp = req_builder.send().await?;
        log::info!("Called GET with path: {}", path);

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
                log::info!("Re auth worked and response was OK");
                Ok(resp)
            }
            _ => {
                log::info!("Response was ok");
                Ok(resp)
            }
        }
    }

    pub fn is_logged_in(&self) -> bool {
        *self.logged_in.lock().unwrap()
    }

    pub async fn get_torrents(&self) -> Result<Vec<Torrent>, Error> {
        Ok(self
            .get(
                "/api/v2/torrents/info?sort=added_on&reverse=true",
                None::<&()>,
            )
            .await?
            .json::<Vec<Torrent>>()
            .await?)
    }

    pub async fn get_torrent_by_hash(&self, hash: impl Into<String>) -> Result<Torrent, Error> {
        let hash = hash.into();

        let torrents = self
            .get("/api/v2/torrents/info", Some(&[("hashes", &hash)]))
            .await?
            .json::<Vec<Torrent>>()
            .await?;

        match torrents.first() {
            Some(t) => Ok(t.to_owned()),
            None => Err(Error::TorrentNotFound(hash)),
        }
    }

    pub async fn queue_torrent(
        &self,
        torrent_bytes: Vec<u8>,
    ) -> Result<TorrentPostResponse, Error> {
        self.ensure_logged_in().await?;

        let url = format!("{}{}", self.base_url, "/api/v2/torrents/add");
        let part = multipart::Part::bytes(torrent_bytes)
            .file_name("torrent.torrent")
            .mime_str("application/x-bittorrent")?;

        let form = multipart::Form::new()
            .part("torrents", part)
            .text("paused", "false");

        let form = if let Some(sp) = &self.save_path {
            form.text("savepath", sp.to_string())
        } else {
            form
        };

        let res = self
            .http
            .post(url)
            .header("Referer", &self.base_url)
            .multipart(form)
            .send()
            .await?
            .error_for_status()?
            .json::<TorrentPostResponse>()
            .await?;

        if res.success_count == 0 || res.added_torrent_ids.is_empty() {
            return Err(Error::Request(format!(
                "qbittorrent did not add the torrent (success_count={}, failure_count={})",
                res.success_count, res.failure_count
            )));
        }

        Ok(res)
    }
}
