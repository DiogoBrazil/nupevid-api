use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ProtectiveMeasuresQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    #[serde(alias = "include_complement_for_entities")]
    pub include_related_entities: Option<bool>,
    pub victim_id: Option<Uuid>,
    pub offender_id: Option<Uuid>,
}
