use serde::Deserialize;

#[derive(Deserialize)]
pub struct UserSearchQuery {
    pub name: Option<String>,
    pub registration: Option<String>,
}
