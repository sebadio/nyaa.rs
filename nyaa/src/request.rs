use crate::{NyaaCategory, filter::NyaaFilter};

// Page filtering doesn't work in rss mode
// maybe scrape the html instaed?
pub struct NyaaRequest {
    pub query: String,
    pub page: i32,
    pub category: NyaaCategory,
    pub filter: NyaaFilter,
}

impl NyaaRequest {
    pub fn new(query: impl Into<String>) -> NyaaRequest {
        NyaaRequest {
            query: query.into(),
            category: NyaaCategory::Anime,
            filter: NyaaFilter::NoFilter,
            page: 1,
        }
    }

    pub fn set_category(mut self, category: NyaaCategory) -> NyaaRequest {
        self.category = category;
        self
    }

    #[expect(dead_code)] // FIX: Do i need to implement html scrape just for this
    pub fn set_page(mut self, page: i32) -> NyaaRequest {
        self.page = page;
        self
    }

    pub fn set_filter(mut self, filter: NyaaFilter) -> NyaaRequest {
        self.filter = filter;
        self
    }

    pub fn to_query_pairs(self) -> Vec<(&'static str, String)> {
        vec![
            ("q", self.query),
            ("c", self.category.as_str().to_string()),
            ("f", self.filter.as_str().to_string()),
            ("p", self.page.to_string()),
        ]
    }
}
