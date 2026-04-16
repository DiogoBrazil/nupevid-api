use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Profile {
    #[serde(rename = "ROOT")]
    Root,
    #[serde(rename = "CITY_ADMIN")]
    CityAdmin,
    #[serde(rename = "CITY_USER")]
    CityUser,
}

impl Profile {
    pub fn as_str(&self) -> &str {
        match self {
            Profile::Root => "ROOT",
            Profile::CityAdmin => "CITY_ADMIN",
            Profile::CityUser => "CITY_USER",
        }
    }
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<&str> for Profile {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "ROOT" => Ok(Profile::Root),
            "CITY_ADMIN" => Ok(Profile::CityAdmin),
            "CITY_USER" => Ok(Profile::CityUser),
            other => Err(format!("Invalid profile: '{}'", other)),
        }
    }
}
