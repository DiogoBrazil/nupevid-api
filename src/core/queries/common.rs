use serde::Deserialize;

#[derive(Deserialize)]
pub struct IncludeComplementQuery {
    pub include_complement_for_entities: Option<bool>,
}
