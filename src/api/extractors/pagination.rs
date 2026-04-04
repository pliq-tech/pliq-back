use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct PaginationParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

impl PaginationParams {
    pub fn into_pagination(self) -> pliq_back_db::models::Pagination {
        pliq_back_db::models::Pagination::new(self.page.unwrap_or(1), self.per_page.unwrap_or(20))
    }
}
