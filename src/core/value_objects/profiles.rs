use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgTypeInfo, PgValueRef};
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

impl sqlx::Type<sqlx::Postgres> for Profile {
    fn type_info() -> PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        <String as sqlx::Type<sqlx::Postgres>>::compatible(ty)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for Profile {
    fn decode(value: PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        Profile::try_from(s.as_str()).map_err(|e| e.into())
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for Profile {
    fn encode_by_ref(
        &self,
        buf: &mut <sqlx::Postgres as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <String as sqlx::Encode<sqlx::Postgres>>::encode(self.as_str().to_string(), buf)
    }
}
