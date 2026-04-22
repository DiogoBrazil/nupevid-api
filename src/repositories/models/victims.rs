use chrono::{DateTime, NaiveDate, Utc};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::core::entities::common::{AddressType, EducationLevel, PhoneType};
use crate::core::entities::victims::{Victim, VictimAddress, VictimPhone};

#[derive(Debug, Clone, FromRow)]
pub struct VictimRow {
    pub id: Uuid,
    pub full_name: String,
    pub cpf: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub city_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
    pub education_level: Option<EducationLevel>,
    pub occupation: Option<String>,
    pub has_children: bool,
    pub children_count: Option<i32>,
    pub is_pregnant: Option<bool>,
    pub has_special_needs: bool,
    pub special_needs_type: Option<Vec<String>>,
    pub uses_alcohol: bool,
    pub uses_drugs: bool,
    pub has_psychiatric_issues: bool,
    pub psychiatric_issues_type: Option<Vec<String>>,
}

impl From<VictimRow> for Victim {
    fn from(row: VictimRow) -> Self {
        Victim {
            id: row.id,
            full_name: row.full_name,
            cpf: row.cpf,
            birth_date: row.birth_date,
            city_id: row.city_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
            is_deleted: row.is_deleted,
            education_level: row.education_level,
            occupation: row.occupation,
            has_children: row.has_children,
            children_count: row.children_count,
            is_pregnant: row.is_pregnant,
            has_special_needs: row.has_special_needs,
            special_needs_type: row.special_needs_type,
            uses_alcohol: row.uses_alcohol,
            uses_drugs: row.uses_drugs,
            has_psychiatric_issues: row.has_psychiatric_issues,
            psychiatric_issues_type: row.psychiatric_issues_type,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct VictimPhoneRow {
    pub id: Uuid,
    pub victim_id: Uuid,
    pub phone: String,
    pub phone_type: Option<PhoneType>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

impl From<VictimPhoneRow> for VictimPhone {
    fn from(row: VictimPhoneRow) -> Self {
        VictimPhone {
            id: row.id,
            victim_id: row.victim_id,
            phone: row.phone,
            phone_type: row.phone_type,
            created_at: row.created_at,
            updated_at: row.updated_at,
            is_deleted: row.is_deleted,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct VictimAddressRow {
    pub id: Uuid,
    pub victim_id: Uuid,
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

impl From<VictimAddressRow> for VictimAddress {
    fn from(row: VictimAddressRow) -> Self {
        VictimAddress {
            id: row.id,
            victim_id: row.victim_id,
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
