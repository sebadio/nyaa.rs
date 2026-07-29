use feed_rs::model::Feed;
use feed_rs::parser;
use reqwest;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum NyaaSearchErrors {
    #[error("failed to fetch rss feed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("failed to parse rss feed: {0}")]
    Parse(#[from] feed_rs::parser::ParseFeedError),
}

#[derive(Debug, Clone)]
pub(crate) struct NyaaSearch {
    http_client: reqwest::Client,
    url: String,
}

impl NyaaSearch {
    pub fn new() -> Self {
        Self {
            http_client: reqwest::Client::new(),
            url: "https://nyaa.si/?page=rss".to_string(),
        }
    }

    pub async fn fetch(&self) -> Result<Feed, NyaaSearchErrors> {
        let resp = self.http_client.get(&self.url).send().await?;
        let bytes = resp.bytes().await?;
        Ok(parser::parse(bytes.as_ref())?)
    }
}
