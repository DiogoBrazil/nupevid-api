use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ViolenceType {
    Physical,
    Sexual,
    Psychological,
    Moral,
}

impl ViolenceType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Physical => "Physical",
            Self::Sexual => "Sexual",
            Self::Psychological => "Psychological",
            Self::Moral => "Moral",
        }
    }
}

impl TryFrom<&str> for ViolenceType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Physical" => Ok(Self::Physical),
            "Sexual" => Ok(Self::Sexual),
            "Psychological" => Ok(Self::Psychological),
            "Moral" => Ok(Self::Moral),
            other => Err(format!("Invalid violence type: '{}'", other)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum RelationshipToVictim {
    #[serde(rename = "Spouse")]
    Spouse,
    #[serde(rename = "Ex-Spouse")]
    ExSpouse,
    #[serde(rename = "Mother")]
    Mother,
    #[serde(rename = "Father")]
    Father,
    #[serde(rename = "Stepfather")]
    Stepfather,
    #[serde(rename = "Stepmother")]
    Stepmother,
    #[serde(rename = "Son/Daughter")]
    SonDaughter,
    #[serde(rename = "Sibling")]
    Sibling,
    #[serde(rename = "Cousin")]
    Cousin,
    #[serde(rename = "Grandfather")]
    Grandfather,
    #[serde(rename = "Grandmother")]
    Grandmother,
    #[serde(rename = "Brother/Sister-in-law")]
    BrotherSisterInLaw,
    #[serde(rename = "Father/Mother-in-law")]
    FatherMotherInLaw,
    #[serde(rename = "Son-in-law")]
    SonInLaw,
    #[serde(rename = "Daughter-in-law")]
    DaughterInLaw,
    #[serde(rename = "Uncle/Aunt")]
    UncleAunt,
}

impl RelationshipToVictim {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Spouse => "Spouse",
            Self::ExSpouse => "Ex-Spouse",
            Self::Mother => "Mother",
            Self::Father => "Father",
            Self::Stepfather => "Stepfather",
            Self::Stepmother => "Stepmother",
            Self::SonDaughter => "Son/Daughter",
            Self::Sibling => "Sibling",
            Self::Cousin => "Cousin",
            Self::Grandfather => "Grandfather",
            Self::Grandmother => "Grandmother",
            Self::BrotherSisterInLaw => "Brother/Sister-in-law",
            Self::FatherMotherInLaw => "Father/Mother-in-law",
            Self::SonInLaw => "Son-in-law",
            Self::DaughterInLaw => "Daughter-in-law",
            Self::UncleAunt => "Uncle/Aunt",
        }
    }
}

impl TryFrom<&str> for RelationshipToVictim {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Spouse" => Ok(Self::Spouse),
            "Ex-Spouse" => Ok(Self::ExSpouse),
            "Mother" => Ok(Self::Mother),
            "Father" => Ok(Self::Father),
            "Stepfather" => Ok(Self::Stepfather),
            "Stepmother" => Ok(Self::Stepmother),
            "Son/Daughter" => Ok(Self::SonDaughter),
            "Sibling" => Ok(Self::Sibling),
            "Cousin" => Ok(Self::Cousin),
            "Grandfather" => Ok(Self::Grandfather),
            "Grandmother" => Ok(Self::Grandmother),
            "Brother/Sister-in-law" => Ok(Self::BrotherSisterInLaw),
            "Father/Mother-in-law" => Ok(Self::FatherMotherInLaw),
            "Son-in-law" => Ok(Self::SonInLaw),
            "Daughter-in-law" => Ok(Self::DaughterInLaw),
            "Uncle/Aunt" => Ok(Self::UncleAunt),
            other => Err(format!("Invalid relationship to victim: '{}'", other)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ProtectiveMeasureStatus {
    Valid,
    Revoked,
    Expired,
}

impl ProtectiveMeasureStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Valid => "Valid",
            Self::Revoked => "Revoked",
            Self::Expired => "Expired",
        }
    }
}

impl TryFrom<&str> for ProtectiveMeasureStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Valid" => Ok(Self::Valid),
            "Revoked" => Ok(Self::Revoked),
            "Expired" => Ok(Self::Expired),
            other => Err(format!("Invalid protective measure status: '{}'", other)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectiveMeasure {
    pub id: Uuid,
    pub process_number: String,
    pub sei_process_number: Option<String>,
    pub occurrence_report_number: Option<String>,
    pub issued_at: NaiveDate,
    pub valid_until: Option<NaiveDate>,
    pub judicial_authority: String,
    pub court_district_id: Uuid,
    pub distance_meters: Option<i32>,
    pub status: ProtectiveMeasureStatus,
    pub violence_types: Vec<ViolenceType>,
    pub relationship_to_victim: RelationshipToVictim,
    pub assaults_children: bool,
    pub was_drunk_during_assault: bool,
    pub victim_id: Uuid,
    pub offender_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectiveMeasureExtension {
    pub id: Uuid,
    pub protective_measure_id: Uuid,
    pub extension_number: i32,
    pub extension_date: NaiveDate,
    pub new_valid_until: Option<NaiveDate>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}
