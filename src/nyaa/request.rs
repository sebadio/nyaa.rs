use crate::nyaa::{NyaaCategory, filter::NyaaFilter};

// Page filtering doesn't work in rss mode
// maybe scrape the html instaed?
pub(crate) struct NyaaRequest {
    pub(crate) query: String,
    pub(crate) page: i32,
    pub(crate) category: NyaaCategory,
    pub(crate) filter: NyaaFilter,
}

impl NyaaRequest {
    pub(crate) fn new(query: impl Into<String>) -> NyaaRequest {
        NyaaRequest {
            query: query.into(),
            category: NyaaCategory::Anime,
            filter: NyaaFilter::NoFilter,
            page: 1,
        }
    }

    pub(crate) fn set_category(mut self, category: NyaaCategory) -> NyaaRequest {
        self.category = category;
        self
    }

    #[expect(dead_code)] // FIX: Do i need to implement html scrape just for this
    pub(crate) fn set_page(mut self, page: i32) -> NyaaRequest {
        self.page = page;
        self
    }

    pub(crate) fn set_filter(mut self, filter: NyaaFilter) -> NyaaRequest {
        self.filter = filter;
        self
    }

    pub(crate) fn to_query_pairs(self) -> Vec<(&'static str, String)> {
        vec![
            ("q", self.query),
            ("c", self.category.as_str().to_string()),
            ("f", self.filter.as_str().to_string()),
            ("p", self.page.to_string()),
        ]
    }
}
