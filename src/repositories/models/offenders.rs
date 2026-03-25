use chrono::{DateTime, NaiveDate, Utc};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::core::entities::common::{AddressType, EducationLevel, PhoneType};
use crate::core::entities::offenders::{Offender, OffenderAddress, OffenderPhone, SecurityForce};

#[derive(Debug, Clone, FromRow)]
pub struct OffenderRow {
    pub id: Uuid,
    pub full_name: String,
    pub cpf: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub city_id: Uuid,
    pub imprisoned: bool,
    pub occupation: Option<String>,
    pub is_public_security_agent: bool,
    pub security_force: Option<SecurityForce>,
    pub uses_alcohol: bool,
    pub uses_drugs: bool,
    pub has_psychiatric_issues: bool,
    pub psychiatric_issues_type: Option<Vec<String>>,
    pub education_level: EducationLevel,
    pub observation: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

impl From<OffenderRow> for Offender {
    fn from(row: OffenderRow) -> Self {
        Offender {
            id: row.id,
            full_name: row.full_name,
            cpf: row.cpf,
            birth_date: row.birth_date,
            city_id: row.city_id,
            imprisoned: row.imprisoned,
            occupation: row.occupation,
            is_public_security_agent: row.is_public_security_agent,
            security_force: row.security_force,
            uses_alcohol: row.uses_alcohol,
            uses_drugs: row.uses_drugs,
            has_psychiatric_issues: row.has_psychiatric_issues,
            psychiatric_issues_type: row.psychiatric_issues_type,
            education_level: row.education_level,
            observation: row.observation,
            created_at: row.created_at,
            updated_at: row.updated_at,
            is_deleted: row.is_deleted,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct OffenderPhoneRow {
    pub id: Uuid,
    pub offender_id: Uuid,
    pub phone: String,
    pub phone_type: Option<PhoneType>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

impl From<OffenderPhoneRow> for OffenderPhone {
    fn from(row: OffenderPhoneRow) -> Self {
        OffenderPhone {
            id: row.id,
            offender_id: row.offender_id,
            phone: row.phone,
            phone_type: row.phone_type,
            created_at: row.created_at,
            updated_at: row.updated_at,
            is_deleted: row.is_deleted,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct OffenderAddressRow {
    pub id: Uuid,
    pub offender_id: Uuid,
    pub street: Option<String>,
    pub number: Option<String>,
    pub district: Option<String>,
    pub city_id: Option<Uuid>,
    pub zip_code: Option<String>,
    pub complement: Option<String>,
    pub address_type: AddressType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

impl From<OffenderAddressRow> for OffenderAddress {
    fn from(row: OffenderAddressRow) -> Self {
        OffenderAddress {
            id: row.id,
            offender_id: row.offender_id,
            street: row.street,
            number: row.number,
            district: row.district,
            city_id: row.city_id,
            zip_code: row.zip_code,
            complement: row.complement,
            address_type: row.address_type,
            created_at: row.created_at,
            updated_at: row.updated_at,
            is_deleted: row.is_deleted,
        }
    }
}
