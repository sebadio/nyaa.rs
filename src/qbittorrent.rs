use reqwest;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Torrent {
    pub name: String,
    pub hash: String,
    pub content_path: String,
    pub size: u64,
    pub progress: f64,
    pub state: String,
    pub num_seeds: u32,
    pub num_leechs: u32,
    pub num_complete: u32,
    pub num_incomplete: u32,
    pub added_on: i64,
}

#[derive(Debug, Clone)]
pub enum Error {
    Auth,
    Http(String),
}

#[derive(Debug, Clone)]
pub struct Client {
    http: reqwest::Client,
    base_url: String,
}

impl Client {
    pub fn new(base_url: impl Into<String>) -> Result<Self, Error> {
        let http = reqwest::Client::builder()
            .cookie_store(true)
            .build()
            .map_err(|e| Error::Http(e.to_string()))?;
        Ok(Self {
            http,
            base_url: base_url.into(),
        })
    }

    pub async fn login(&self, user: &str, pass: &str) -> Result<(), Error> {
        let resp = self
            .http
            .post(format!("{}/api/v2/auth/login", self.base_url))
            .header("Referer", &self.base_url)
            .form(&[("username", user), ("password", pass)])
            .send()
            .await
            .map_err(|e| Error::Http(e.to_string()))?;

        if resp.status().is_success() {
            Ok(())
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            Err(Error::Http(format!("login {status}: {}", body.trim())))
        }
    }

    pub async fn get_torrents(&self) -> Result<Vec<Torrent>, Error> {
        self.http
            .get(format!(
                "{}/api/v2/torrents/info?sort=added_on&reverse=true",
                self.base_url
            ))
            .send()
            .await
            .map_err(|e| Error::Http(e.to_string()))?
            .json::<Vec<Torrent>>()
            .await
            .map_err(|e| Error::Http(e.to_string()))
    }
}
