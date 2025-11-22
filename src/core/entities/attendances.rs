use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate, NaiveTime};

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateAttendance {
    pub victim_id: Uuid,
    pub was_victim_present: bool,
    pub attendance_date: NaiveDate,
    pub attendance_time: NaiveTime,
    pub description: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateAttendanceAddress {
    pub attendance_id: Uuid,
    pub street: Option<String>,
    pub number: Option<String>,
    pub district: Option<String>,
    pub city_name: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
    pub complement: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateAttendanceAddress {
    pub street: Option<String>,
    pub number: Option<String>,
    pub district: Option<String>,
    pub city_name: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
    pub complement: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AttendanceAddress {
    pub id: Uuid,
    pub attendance_id: Uuid,
    pub street: Option<String>,
    pub number: Option<String>,
    pub district: Option<String>,
    pub city_name: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
    pub complement: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}
