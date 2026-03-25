use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::entities::common::{AddressData, EducationLevel, PhoneData};
use crate::core::entities::offenders::SecurityForce;

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateOffender {
    pub full_name: String,
    pub cpf: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub city_id: Option<Uuid>,
    pub imprisoned: bool,
    pub occupation: Option<String>,
    #[serde(default, skip_deserializing)]
    pub is_public_security_agent: bool,
    pub security_force: Option<SecurityForce>,
    pub uses_alcohol: bool,
    pub uses_drugs: bool,
    #[serde(default)]
    pub has_psychiatric_issues: bool,
    pub psychiatric_issues_type: Option<Vec<String>>,
    pub education_level: EducationLevel,
    pub observation: Option<String>,
    pub phones: Option<Vec<PhoneData>>,
    pub addresses: Option<Vec<AddressData>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateOffender {
    pub full_name: String,
    pub cpf: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub city_id: Option<Uuid>,
    pub imprisoned: bool,
    pub occupation: Option<String>,
    #[serde(default, skip_deserializing)]
    pub is_public_security_agent: bool,
    pub security_force: Option<SecurityForce>,
    pub uses_alcohol: bool,
    pub uses_drugs: bool,
    #[serde(default)]
    pub has_psychiatric_issues: bool,
    pub psychiatric_issues_type: Option<Vec<String>>,
    pub education_level: EducationLevel,
    pub observation: Option<String>,
    pub phones: Option<Vec<PhoneData>>,
    pub addresses: Option<Vec<AddressData>>,
}
