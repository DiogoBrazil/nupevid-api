use serde::Deserialize;

#[derive(Deserialize)]
pub struct ProtectiveMeasuresQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub include_related_entities: Option<bool>,
}
