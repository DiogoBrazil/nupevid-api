use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateCity {
    pub name: String,
    pub state: String,
    pub battalion: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateCity {
    pub name: String,
    pub state: String,
    pub battalion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct City {
    pub id: Uuid,
    pub name: String,
    pub state: String,
    pub battalion: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CityComplement {
    pub id: Uuid,
    pub name: String,
    pub state: String,
    pub battalion: String,
    pub is_deleted: bool,
}

impl From<City> for CityComplement {
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
