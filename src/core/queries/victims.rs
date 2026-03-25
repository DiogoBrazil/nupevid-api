use serde::Deserialize;

#[derive(Deserialize)]
pub struct VictimSearchQuery {
    pub name: Option<String>,
    pub cpf: Option<String>,
}
