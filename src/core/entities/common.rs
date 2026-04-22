use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum PhoneType {
    #[serde(rename = "Mobile")]
    Mobile,
    #[serde(rename = "Residential")]
    Residential,
    #[serde(rename = "Work")]
    Work,
}

impl PhoneType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Mobile => "Mobile",
            Self::Residential => "Residential",
            Self::Work => "Work",
        }
    }
}

impl TryFrom<&str> for PhoneType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Mobile" => Ok(Self::Mobile),
            "Residential" => Ok(Self::Residential),
            "Work" => Ok(Self::Work),
            other => Err(format!("Invalid phone type: '{}'", other)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum AddressType {
    Residential,
    Work,
    Correspondence,
    Commercial,
    Institutional,
    Temporary,
    Other,
}

impl AddressType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Residential => "Residential",
            Self::Work => "Work",
            Self::Correspondence => "Correspondence",
            Self::Commercial => "Commercial",
            Self::Institutional => "Institutional",
            Self::Temporary => "Temporary",
            Self::Other => "Other",
        }
    }
}

impl TryFrom<&str> for AddressType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Residential" => Ok(Self::Residential),
            "Work" => Ok(Self::Work),
            "Correspondence" => Ok(Self::Correspondence),
            "Commercial" => Ok(Self::Commercial),
            "Institutional" => Ok(Self::Institutional),
            "Temporary" => Ok(Self::Temporary),
            "Other" => Ok(Self::Other),
            other => Err(format!("Invalid address type: '{}'", other)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum EducationLevel {
    #[serde(rename = "Elementary")]
    Elementary,
    #[serde(rename = "High School")]
    HighSchool,
    #[serde(rename = "College")]
    College,
    #[serde(rename = "Postgraduate")]
    Postgraduate,
    #[serde(rename = "Illiterate")]
    Illiterate,
    #[serde(rename = "Semi-illiterate")]
    SemiIlliterate,
    #[serde(rename = "Master")]
    Master,
    #[serde(rename = "Doctorate")]
    Doctorate,
}

impl EducationLevel {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Elementary => "Elementary",
            Self::HighSchool => "High School",
            Self::College => "College",
            Self::Postgraduate => "Postgraduate",
            Self::Illiterate => "Illiterate",
            Self::SemiIlliterate => "Semi-illiterate",
            Self::Master => "Master",
            Self::Doctorate => "Doctorate",
        }
    }
}

impl TryFrom<&str> for EducationLevel {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Elementary" => Ok(Self::Elementary),
            "High School" => Ok(Self::HighSchool),
            "College" => Ok(Self::College),
            "Postgraduate" => Ok(Self::Postgraduate),
            "Illiterate" => Ok(Self::Illiterate),
            "Semi-illiterate" => Ok(Self::SemiIlliterate),
            "Master" => Ok(Self::Master),
            "Doctorate" => Ok(Self::Doctorate),
            other => Err(format!("Invalid education level: '{}'", other)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PhoneData {
    pub phone: String,
    pub phone_type: Option<PhoneType>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AddressData {
    pub street: Option<String>,
    pub number: Option<String>,
    pub district: Option<String>,
    pub city_id: Uuid,
    pub zip_code: Option<String>,
    pub complement: Option<String>,
    pub address_type: AddressType,
}

/// Normalize boolean flag from optional list presence
pub fn derive_flag_from_list(values: &Option<Vec<String>>) -> (bool, Option<Vec<String>>) {
    match values {
        Some(list) if !list.is_empty() => (true, Some(list.clone())),
        _ => (false, None),
    }
}

/// Derive security agent flag from the presence of a security force value
pub fn is_security_agent<T>(security_force: &Option<T>) -> bool {
    security_force.is_some()
}
