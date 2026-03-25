use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::entities::common::{AddressData, EducationLevel, PhoneData};

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateVictim {
    pub full_name: String,
    pub cpf: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub city_id: Option<Uuid>,
    pub phones: Option<Vec<PhoneData>>,
    pub addresses: Option<Vec<AddressData>>,
    pub education_level: Option<EducationLevel>,
    pub occupation: Option<String>,
    #[serde(default)]
    pub has_children: bool,
    pub children_count: Option<i32>,
    pub is_pregnant: Option<bool>,
    #[serde(default)]
    pub has_special_needs: bool,
    pub special_needs_type: Option<Vec<String>>,
    pub uses_alcohol: bool,
    pub uses_drugs: bool,
    #[serde(default)]
    pub has_psychiatric_issues: bool,
    pub psychiatric_issues_type: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateVictim {
    pub full_name: String,
    pub cpf: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub city_id: Option<Uuid>,
    pub phones: Option<Vec<PhoneData>>,
    pub addresses: Option<Vec<AddressData>>,
    pub education_level: Option<EducationLevel>,
    pub occupation: Option<String>,
    #[serde(default)]
    pub has_children: bool,
    pub children_count: Option<i32>,
    pub is_pregnant: Option<bool>,
    #[serde(default)]
    pub has_special_needs: bool,
    pub special_needs_type: Option<Vec<String>>,
    pub uses_alcohol: bool,
    pub uses_drugs: bool,
    #[serde(default)]
    pub has_psychiatric_issues: bool,
    pub psychiatric_issues_type: Option<Vec<String>>,
}
