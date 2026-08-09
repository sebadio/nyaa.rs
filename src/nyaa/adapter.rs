use crate::nyaa::{NyaaItem, NyaaRss, request::NyaaRequest};
use log::info;
use quick_xml;
use reqwest;
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub(crate) enum NyaaAdapterError {
    #[error("failed to fetch rss feed: {0}")]
    Request(String),

    #[error("failed to parse rss feed: {0}")]
    Parsing(#[from] quick_xml::DeError),

    #[error("Failed to write to temp dir: {0}")]
    Io(String),
}

impl From<std::io::Error> for NyaaAdapterError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(format!("IO Error: {:#?}", value.to_string()))
    }
}

impl From<reqwest::Error> for NyaaAdapterError {
    fn from(e: reqwest::Error) -> Self {
        Self::Request(format!(
            "reqwest error: {err:#?} \nURL :{url:#?}\nStatus: {status:#?}",
            err = e.to_string(),
            url = e.url(),
            status = e.status()
        ))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct NyaaItemBytes {
    pub(crate) item: NyaaItem,
    pub(crate) bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub(crate) struct NyaaAdapter {
    http_client: reqwest::Client,
    url: String,
}

impl NyaaAdapter {
    pub(crate) fn new() -> Result<Self, NyaaAdapterError> {
        let http = reqwest::Client::builder().build()?;
        Ok(Self {
            http_client: http,
            url: "https://nyaa.si/?page=rss".into(),
        })
    }

    pub(crate) async fn download_torrent(
        &self,
        item: NyaaItem,
    ) -> Result<NyaaItemBytes, NyaaAdapterError> {
        info!("Starting download of .torrent: {} ", item.title);
        let resp = self.http_client.get(&item.link).send().await?;
        Ok(NyaaItemBytes {
            item,
            bytes: resp.bytes().await?.to_vec(),
        })
    }

    pub(crate) async fn fetch(
        &self,
        request: Option<NyaaRequest>,
    ) -> Result<Vec<NyaaItem>, NyaaAdapterError> {
        let mut builder = self.http_client.get(&self.url);

        if let Some(req) = request {
            builder = builder.query(&req.to_query_pairs());
        }

        let resp = builder.send().await?;
        let bytes = resp.bytes().await?;
        let rss: NyaaRss = quick_xml::de::from_reader(bytes.as_ref())?;
        Ok(rss.channel.items)
    }
}
