use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ResetUserPasswordResponse {
    pub temporary_password: String,
}
