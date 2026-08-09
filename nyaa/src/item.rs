use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct NyaaItem {
    pub title: String,
    pub link: String,
    pub guid: Guid,
    #[serde(rename = "pubDate")]
    pub pub_date: String,
    pub seeders: u32,
    pub leechers: u32,
    pub downloads: u32,
    #[serde(rename = "infoHash")]
    pub info_hash: String,
    #[serde(rename = "categoryId")]
    pub category_id: String,
    pub category: String,
    pub size: String,
    pub comments: u32,
    #[serde(deserialize_with = "yes_no_bool")]
    pub trusted: bool,
    #[serde(deserialize_with = "yes_no_bool")]
    pub remake: bool,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Guid {
    #[serde(rename = "@isPermaLink")]
    pub is_permalink: bool,
    #[serde(rename = "$text")]
    pub value: String,
}

fn yes_no_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.eq_ignore_ascii_case("yes"))
}

#[derive(Debug, Clone, Deserialize)]
pub struct RssChannel {
    #[serde(rename = "item", default)]
    pub items: Vec<NyaaItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NyaaRss {
    pub channel: RssChannel,
}
