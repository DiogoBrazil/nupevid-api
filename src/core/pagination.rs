#[derive(Debug)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub page: i64,
    pub page_size: i64,
    pub total_items: i64,
}
