use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AttendanceAddressData {
    pub street: Option<String>,
    pub number: Option<String>,
    pub district: Option<String>,
    pub city_id: Option<Uuid>,
    pub zip_code: Option<String>,
    pub complement: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateAttendance {
    pub victim_id: Uuid,
    pub was_victim_present: bool,
    pub attendance_date: NaiveDate,
    pub attendance_time: NaiveTime,
    pub description: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub address: Option<AttendanceAddressData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateAttendance {
    pub victim_id: Uuid,
    pub was_victim_present: bool,
    pub attendance_date: NaiveDate,
    pub attendance_time: NaiveTime,
    pub description: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub address: Option<AttendanceAddressData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Attendance {
    pub id: Uuid,
    pub victim_id: Uuid,
    pub was_victim_present: bool,
    pub attendance_date: NaiveDate,
    pub attendance_time: NaiveTime,
    pub description: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceAddressResponse {
    pub id: Uuid,
    pub street: Option<String>,
    pub number: Option<String>,
    pub district: Option<String>,
    pub city_id: Option<Uuid>,
    pub zip_code: Option<String>,
    pub complement: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceWithAddress {
    pub id: Uuid,
    pub victim_id: Uuid,
    pub was_victim_present: bool,
    pub attendance_date: NaiveDate,
    pub attendance_time: NaiveTime,
    pub description: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
    pub address: Option<AttendanceAddressResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AttendanceAddress {
    pub id: Uuid,
    pub attendance_id: Uuid,
    pub street: Option<String>,
    pub number: Option<String>,
    pub district: Option<String>,
    pub city_id: Option<Uuid>,
    pub zip_code: Option<String>,
    pub complement: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

impl AttendanceAddress {
    pub fn to_response(self) -> AttendanceAddressResponse {
        AttendanceAddressResponse {
            id: self.id,
            street: self.street,
            number: self.number,
            district: self.district,
            city_id: self.city_id,
            zip_code: self.zip_code,
            complement: self.complement,
        }
    }
}

impl Attendance {
    pub fn with_address(self, address: Option<AttendanceAddress>) -> AttendanceWithAddress {
        AttendanceWithAddress {
            id: self.id,
            victim_id: self.victim_id,
            was_victim_present: self.was_victim_present,
            attendance_date: self.attendance_date,
            attendance_time: self.attendance_time,
            description: self.description,
            latitude: self.latitude,
            longitude: self.longitude,
            created_at: self.created_at,
            updated_at: self.updated_at,
            is_deleted: self.is_deleted,
            address: address.map(|a| a.to_response()),
        }
    }
}
