use serde::{Deserialize, Serialize};

/// Khớp Java: PageLink — pagination parameters
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PageLink {
    pub page: i64,
    pub page_size: i64,
    pub text_search: Option<String>,
    pub sort_property: Option<String>,
    pub sort_order: SortOrder,
}

impl PageLink {
    pub fn new(page: i64, page_size: i64) -> Self {
        Self {
            page,
            page_size: page_size.min(1000), // Giới hạn tối đa như TB Java
            text_search: None,
            sort_property: Some("createdTime".into()),
            sort_order: SortOrder::Desc,
        }
    }

    pub fn offset(&self) -> i64 {
        self.page * self.page_size
    }
}

/// Khớp Java: PageData<T>
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageData<T> {
    pub data: Vec<T>,
    pub total_pages: i64,
    pub total_elements: i64,
    pub has_next: bool,
}

impl<T> PageData<T> {
    pub fn new(data: Vec<T>, total_elements: i64, page_link: &PageLink) -> Self {
        let total_pages = if page_link.page_size == 0 {
            0
        } else {
            (total_elements + page_link.page_size - 1) / page_link.page_size
        };
        let has_next = page_link.page + 1 < total_pages;
        Self { data, total_pages, total_elements, has_next }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum SortOrder {
    #[default]
    Desc,
    Asc,
}

impl SortOrder {
    pub fn as_sql(&self) -> &'static str {
        match self {
            SortOrder::Desc => "DESC",
            SortOrder::Asc  => "ASC",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── PageLink ──────────────────────────────────────────────────────────────

    #[test]
    fn offset_first_page() {
        assert_eq!(PageLink::new(0, 20).offset(), 0);
    }

    #[test]
    fn offset_second_page() {
        assert_eq!(PageLink::new(1, 20).offset(), 20);
    }

    #[test]
    fn offset_third_page() {
        assert_eq!(PageLink::new(2, 10).offset(), 20);
    }

    #[test]
    fn page_size_capped_at_1000() {
        let p = PageLink::new(0, 9999);
        assert_eq!(p.page_size, 1000);
    }

    #[test]
    fn page_size_zero_stays_zero() {
        let p = PageLink::new(0, 0);
        assert_eq!(p.page_size, 0);
        assert_eq!(p.offset(), 0);
    }

    #[test]
    fn default_sort_is_desc() {
        let p = PageLink::new(0, 10);
        assert_eq!(p.sort_order, SortOrder::Desc);
    }

    // ── PageData ─────────────────────────────────────────────────────────────

    #[test]
    fn total_pages_rounds_up() {
        // 57 items at 20/page → ceil(57/20) = 3
        let pd = PageData::new(Vec::<i32>::new(), 57, &PageLink::new(0, 20));
        assert_eq!(pd.total_pages, 3);
        assert_eq!(pd.total_elements, 57);
    }

    #[test]
    fn has_next_true_when_not_last_page() {
        let pd = PageData::new(Vec::<i32>::new(), 57, &PageLink::new(0, 20));
        assert!(pd.has_next); // page 0 of 3
    }

    #[test]
    fn has_next_false_on_last_page() {
        let pd = PageData::new(Vec::<i32>::new(), 57, &PageLink::new(2, 20));
        assert!(!pd.has_next); // page 2 is last (0-indexed)
    }

    #[test]
    fn has_next_false_middle_page_with_two_total() {
        // 40 items at 20/page → 2 pages (0 and 1)
        let pd = PageData::new(Vec::<i32>::new(), 40, &PageLink::new(1, 20));
        assert_eq!(pd.total_pages, 2);
        assert!(!pd.has_next);
    }

    #[test]
    fn empty_result_set() {
        let pd = PageData::new(Vec::<i32>::new(), 0, &PageLink::new(0, 20));
        assert_eq!(pd.total_pages, 0);
        assert_eq!(pd.total_elements, 0);
        assert!(!pd.has_next);
    }

    #[test]
    fn exact_multiple_total_pages() {
        // 40 items at 20/page → exactly 2 pages
        let pd = PageData::new(Vec::<i32>::new(), 40, &PageLink::new(0, 20));
        assert_eq!(pd.total_pages, 2);
    }

    #[test]
    fn single_item_single_page() {
        let pd = PageData::new(vec![42], 1, &PageLink::new(0, 20));
        assert_eq!(pd.total_pages, 1);
        assert!(!pd.has_next);
    }

    // ── SortOrder ─────────────────────────────────────────────────────────────

    #[test]
    fn sort_order_sql_strings() {
        assert_eq!(SortOrder::Asc.as_sql(), "ASC");
        assert_eq!(SortOrder::Desc.as_sql(), "DESC");
    }
}
