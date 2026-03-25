use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CityComplement {
    pub id: Uuid,
    pub name: String,
    pub state: String,
    pub battalion: String,
    pub is_deleted: bool,
}
