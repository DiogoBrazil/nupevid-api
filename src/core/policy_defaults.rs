use std::collections::HashMap;
use uuid::Uuid;

use crate::core::value_objects::policies::Policy;
use crate::core::value_objects::profiles::Profile;

pub fn default_for_profile(profile: &Profile, city_id: Option<Uuid>) -> HashMap<Policy, Vec<Uuid>> {
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
