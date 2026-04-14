use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgTypeInfo, PgValueRef};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Rank {
    #[serde(rename = "CEL PM")]
    CelPm,
    #[serde(rename = "TC PM")]
    TcPm,
    #[serde(rename = "MAJ PM")]
    MajPm,
    #[serde(rename = "CAP PM")]
    CapPm,
    #[serde(rename = "1º TEN PM")]
    PrimeiroTenPm,
    #[serde(rename = "2º TEN PM")]
    SegundoTenPm,
    #[serde(rename = "ASP OF PM")]
    AspOfPm,
    #[serde(rename = "CAD PM")]
    CadPm,
    #[serde(rename = "ST PM")]
    StPm,
    #[serde(rename = "1º SGT PM")]
    PrimeiroSgtPm,
    #[serde(rename = "2º SGT PM")]
    SegundoSgtPm,
    #[serde(rename = "3º SGT PM")]
    TerceiroSgtPm,
    #[serde(rename = "CB PM")]
    CbPm,
    #[serde(rename = "SD PM")]
    SdPm,
}

impl Rank {
    pub fn as_str(&self) -> &str {
        match self {
            Rank::CelPm => "CEL PM",
            Rank::TcPm => "TC PM",
            Rank::MajPm => "MAJ PM",
            Rank::CapPm => "CAP PM",
            Rank::PrimeiroTenPm => "1º TEN PM",
            Rank::SegundoTenPm => "2º TEN PM",
            Rank::AspOfPm => "ASP OF PM",
            Rank::CadPm => "CAD PM",
            Rank::StPm => "ST PM",
            Rank::PrimeiroSgtPm => "1º SGT PM",
            Rank::SegundoSgtPm => "2º SGT PM",
            Rank::TerceiroSgtPm => "3º SGT PM",
            Rank::CbPm => "CB PM",
            Rank::SdPm => "SD PM",
        }
    }
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<&str> for Rank {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "CEL PM" => Ok(Rank::CelPm),
            "TC PM" => Ok(Rank::TcPm),
            "MAJ PM" => Ok(Rank::MajPm),
            "CAP PM" => Ok(Rank::CapPm),
            "1º TEN PM" => Ok(Rank::PrimeiroTenPm),
            "2º TEN PM" => Ok(Rank::SegundoTenPm),
            "ASP OF PM" => Ok(Rank::AspOfPm),
            "CAD PM" => Ok(Rank::CadPm),
            "ST PM" => Ok(Rank::StPm),
            "1º SGT PM" => Ok(Rank::PrimeiroSgtPm),
            "2º SGT PM" => Ok(Rank::SegundoSgtPm),
            "3º SGT PM" => Ok(Rank::TerceiroSgtPm),
            "CB PM" => Ok(Rank::CbPm),
            "SD PM" => Ok(Rank::SdPm),
            other => Err(format!("Invalid rank: '{}'", other)),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for Rank {
    fn type_info() -> PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        <String as sqlx::Type<sqlx::Postgres>>::compatible(ty)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for Rank {
    fn decode(value: PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        Rank::try_from(s.as_str()).map_err(|e| e.into())
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for Rank {
    fn encode_by_ref(
        &self,
        buf: &mut <sqlx::Postgres as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <String as sqlx::Encode<sqlx::Postgres>>::encode(self.as_str().to_string(), buf)
    }
}
