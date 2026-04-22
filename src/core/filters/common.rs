use serde::Deserialize;

#[derive(Deserialize)]
pub struct IncludeRelatedQuery {
    pub include_related_entities: Option<bool>,
}
