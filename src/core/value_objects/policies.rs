use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

use crate::core::value_objects::profiles::Profile;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Policy {
    // Cities
    #[serde(rename = "create_cities")]
    CreateCities,
    #[serde(rename = "read_cities")]
    ReadCities,
    #[serde(rename = "update_cities")]
    UpdateCities,
    #[serde(rename = "delete_cities")]
    DeleteCities,

    // Users
    #[serde(rename = "create_users")]
    CreateUsers,
    #[serde(rename = "read_users")]
    ReadUsers,
    #[serde(rename = "update_users")]
    UpdateUsers,
    #[serde(rename = "delete_users")]
    DeleteUsers,

    // Victims
    #[serde(rename = "create_victims")]
    CreateVictims,
    #[serde(rename = "read_victims")]
    ReadVictims,
    #[serde(rename = "update_victims")]
    UpdateVictims,
    #[serde(rename = "delete_victims")]
    DeleteVictims,

    // Offenders
    #[serde(rename = "create_offenders")]
    CreateOffenders,
    #[serde(rename = "read_offenders")]
    ReadOffenders,
    #[serde(rename = "update_offenders")]
    UpdateOffenders,
    #[serde(rename = "delete_offenders")]
    DeleteOffenders,

    // Attendances
    #[serde(rename = "create_attendances")]
    CreateAttendances,
    #[serde(rename = "read_attendances")]
    ReadAttendances,
    #[serde(rename = "update_attendances")]
    UpdateAttendances,
    #[serde(rename = "delete_attendances")]
    DeleteAttendances,

    // Protective Measures
    #[serde(rename = "create_protective_measures")]
    CreateProtectiveMeasures,
    #[serde(rename = "read_protective_measures")]
    ReadProtectiveMeasures,
    #[serde(rename = "update_protective_measures")]
    UpdateProtectiveMeasures,
    #[serde(rename = "delete_protective_measures")]
    DeleteProtectiveMeasures,

    // Work Sessions
    #[serde(rename = "create_work_sessions")]
    CreateWorkSessions,
    #[serde(rename = "update_work_sessions")]
    UpdateWorkSessions,
    #[serde(rename = "end_work_sessions")]
    EndWorkSessions,
    #[serde(rename = "view_other_work_sessions")]
    ViewOtherWorkSessions,

    // Attendance Members
    #[serde(rename = "manage_attendance_members")]
    ManageAttendanceMembers,
}

impl Policy {
    pub fn as_str(&self) -> &str {
        match self {
            Policy::CreateCities => "create_cities",
            Policy::ReadCities => "read_cities",
            Policy::UpdateCities => "update_cities",
            Policy::DeleteCities => "delete_cities",
            Policy::CreateUsers => "create_users",
            Policy::ReadUsers => "read_users",
            Policy::UpdateUsers => "update_users",
            Policy::DeleteUsers => "delete_users",
            Policy::CreateVictims => "create_victims",
            Policy::ReadVictims => "read_victims",
            Policy::UpdateVictims => "update_victims",
            Policy::DeleteVictims => "delete_victims",
            Policy::CreateOffenders => "create_offenders",
            Policy::ReadOffenders => "read_offenders",
            Policy::UpdateOffenders => "update_offenders",
            Policy::DeleteOffenders => "delete_offenders",
            Policy::CreateAttendances => "create_attendances",
            Policy::ReadAttendances => "read_attendances",
            Policy::UpdateAttendances => "update_attendances",
            Policy::DeleteAttendances => "delete_attendances",
            Policy::CreateProtectiveMeasures => "create_protective_measures",
            Policy::ReadProtectiveMeasures => "read_protective_measures",
            Policy::UpdateProtectiveMeasures => "update_protective_measures",
            Policy::DeleteProtectiveMeasures => "delete_protective_measures",
            Policy::CreateWorkSessions => "create_work_sessions",
            Policy::UpdateWorkSessions => "update_work_sessions",
            Policy::EndWorkSessions => "end_work_sessions",
            Policy::ViewOtherWorkSessions => "view_other_work_sessions",
            Policy::ManageAttendanceMembers => "manage_attendance_members",
        }
    }

    pub fn is_assignable(&self) -> bool {
        !matches!(
            self,
            Policy::CreateCities | Policy::UpdateCities | Policy::DeleteCities
        )
    }

    pub fn all() -> &'static [Policy] {
        &[
            Policy::CreateCities,
            Policy::ReadCities,
            Policy::UpdateCities,
            Policy::DeleteCities,
            Policy::CreateUsers,
            Policy::ReadUsers,
            Policy::UpdateUsers,
            Policy::DeleteUsers,
            Policy::CreateVictims,
            Policy::ReadVictims,
            Policy::UpdateVictims,
            Policy::DeleteVictims,
            Policy::CreateOffenders,
            Policy::ReadOffenders,
            Policy::UpdateOffenders,
            Policy::DeleteOffenders,
            Policy::CreateAttendances,
            Policy::ReadAttendances,
            Policy::UpdateAttendances,
            Policy::DeleteAttendances,
            Policy::CreateProtectiveMeasures,
            Policy::ReadProtectiveMeasures,
            Policy::UpdateProtectiveMeasures,
            Policy::DeleteProtectiveMeasures,
            Policy::CreateWorkSessions,
            Policy::UpdateWorkSessions,
            Policy::EndWorkSessions,
            Policy::ViewOtherWorkSessions,
            Policy::ManageAttendanceMembers,
        ]
    }

    pub fn default_for_profile(
        profile: &Profile,
        city_id: Option<Uuid>,
    ) -> HashMap<Policy, Vec<Uuid>> {
        let mut policies: HashMap<Policy, Vec<Uuid>> = HashMap::new();

        let policy_list: &[Policy] = match profile {
            Profile::Root => return policies,
            Profile::CityAdmin => &[
                Policy::ReadCities,
                Policy::CreateUsers,
                Policy::ReadUsers,
                Policy::UpdateUsers,
                Policy::DeleteUsers,
                Policy::CreateVictims,
                Policy::ReadVictims,
                Policy::UpdateVictims,
                Policy::DeleteVictims,
                Policy::CreateOffenders,
                Policy::ReadOffenders,
                Policy::UpdateOffenders,
                Policy::DeleteOffenders,
                Policy::CreateAttendances,
                Policy::ReadAttendances,
                Policy::UpdateAttendances,
                Policy::DeleteAttendances,
                Policy::CreateProtectiveMeasures,
                Policy::ReadProtectiveMeasures,
                Policy::UpdateProtectiveMeasures,
                Policy::DeleteProtectiveMeasures,
                Policy::CreateWorkSessions,
                Policy::UpdateWorkSessions,
                Policy::EndWorkSessions,
                Policy::ViewOtherWorkSessions,
                Policy::ManageAttendanceMembers,
            ],
            Profile::CityUser => &[
                Policy::ReadCities,
                Policy::ReadVictims,
                Policy::ReadOffenders,
                Policy::ReadAttendances,
                Policy::ReadProtectiveMeasures,
                Policy::ReadUsers,
                Policy::CreateWorkSessions,
                Policy::UpdateWorkSessions,
                Policy::EndWorkSessions,
                Policy::ManageAttendanceMembers,
            ],
        };

        if let Some(cid) = city_id {
            for policy in policy_list {
                policies.insert(policy.clone(), vec![cid]);
            }
        }

        policies
    }
}

impl fmt::Display for Policy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
