use chrono::NaiveDate;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct ListWorkSessionsQuery {
    pub user_id: Option<Uuid>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub city_id: Option<Uuid>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub include_complement_for_entities: Option<bool>,
}
