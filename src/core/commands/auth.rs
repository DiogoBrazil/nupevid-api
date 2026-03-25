use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Login {
    pub email: String,
    pub password: String,
    #[serde(default = "default_auto_create_session")]
    pub auto_create_session: bool,
}

fn default_auto_create_session() -> bool {
    true
}
