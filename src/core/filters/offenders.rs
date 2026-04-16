use serde::Deserialize;

#[derive(Deserialize)]
pub struct OffenderSearchQuery {
    pub name: Option<String>,
    pub cpf: Option<String>,
}
