use serde::Serialize;
use typeshare::typeshare;

#[derive(Debug, Clone, Copy, Serialize, Default, FromForm)]
#[typeshare]
#[serde(rename_all = "camelCase")]
pub struct Pagination {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub total_pages: Option<u32>,
}

pub const DEFAULT_PAGE: u32 = 1;
pub const DEFAULT_PER_PAGE: u32 = 20;

pub const MIN_PER_PAGE: u32 = 1;
pub const MAX_PER_PAGE: u32 = 250;

impl Pagination {
    pub fn page(&self) -> u64 {
        self.page.unwrap_or(DEFAULT_PAGE).into()
    }

    pub fn per_page(&self) -> u64 {
        self.per_page
            .unwrap_or(DEFAULT_PER_PAGE)
            .clamp(MIN_PER_PAGE, MAX_PER_PAGE)
            .into()
    }

    pub fn offset(&self) -> u64 {
        (self.page() - 1) * self.per_page()
    }

    pub fn with_defaults(mut self) -> Self {
        self.page = self.page.or(Some(DEFAULT_PAGE));
        self.per_page = self.per_page.or(Some(DEFAULT_PER_PAGE));

        self
    }

    pub fn set_total_pages(&mut self, total_items: u64) {
        self.total_pages = Some(self.calculate_total_pages(total_items).try_into().unwrap());
    }

    pub fn calculate_total_pages(&self, total_items: u64) -> u64 {
        (total_items / self.per_page()) + u64::from(total_items % self.per_page() > 0)
    }
}

#[derive(Debug, Serialize)]
#[typeshare]
#[serde(rename_all = "camelCase")]
pub struct PaginationResponse<T> {
    pub items: T,
    pub pagination: Pagination,
}
