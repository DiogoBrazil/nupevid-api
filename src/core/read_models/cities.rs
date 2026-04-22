use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::entities::cities::City;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitySummary {
    pub id: Uuid,
    pub name: String,
    pub state: String,
    pub battalion: String,
    pub is_deleted: bool,
}

impl From<City> for CitySummary {
    fn from(city: City) -> Self {
        Self {
            id: city.id,
            name: city.name,
            state: city.state,
            battalion: city.battalion,
            is_deleted: city.is_deleted,
        }
    }
}
