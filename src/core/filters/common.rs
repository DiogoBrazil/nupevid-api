use serde::Deserialize;

#[derive(Deserialize)]
pub struct IncludeRelatedQuery {
    #[serde(alias = "include_complement_for_entities")]
    pub include_related_entities: Option<bool>,
}
