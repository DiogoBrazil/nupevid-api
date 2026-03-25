use std::collections::HashMap;
use uuid::Uuid;

// Profile constants
pub const PROFILE_ROOT: &str = "ROOT";
pub const PROFILE_CITY_ADMIN: &str = "CITY_ADMIN";
pub const PROFILE_CITY_USER: &str = "CITY_USER";

pub const VALID_PROFILES: [&str; 3] = [PROFILE_ROOT, PROFILE_CITY_ADMIN, PROFILE_CITY_USER];

// Cities policies
pub const POLICY_CREATE_CITIES: &str = "create_cities";
pub const POLICY_READ_CITIES: &str = "read_cities";
pub const POLICY_UPDATE_CITIES: &str = "update_cities";
pub const POLICY_DELETE_CITIES: &str = "delete_cities";

// Users policies
pub const POLICY_CREATE_USERS: &str = "create_users";
pub const POLICY_READ_USERS: &str = "read_users";
pub const POLICY_UPDATE_USERS: &str = "update_users";
pub const POLICY_DELETE_USERS: &str = "delete_users";

// Victims policies
pub const POLICY_CREATE_VICTIMS: &str = "create_victims";
pub const POLICY_READ_VICTIMS: &str = "read_victims";
pub const POLICY_UPDATE_VICTIMS: &str = "update_victims";
pub const POLICY_DELETE_VICTIMS: &str = "delete_victims";

// Offenders policies
pub const POLICY_CREATE_OFFENDERS: &str = "create_offenders";
pub const POLICY_READ_OFFENDERS: &str = "read_offenders";
pub const POLICY_UPDATE_OFFENDERS: &str = "update_offenders";
pub const POLICY_DELETE_OFFENDERS: &str = "delete_offenders";

// Attendances policies
pub const POLICY_CREATE_ATTENDANCES: &str = "create_attendances";
pub const POLICY_READ_ATTENDANCES: &str = "read_attendances";
pub const POLICY_UPDATE_ATTENDANCES: &str = "update_attendances";
pub const POLICY_DELETE_ATTENDANCES: &str = "delete_attendances";

// Protective Measures policies
pub const POLICY_CREATE_PROTECTIVE_MEASURES: &str = "create_protective_measures";
pub const POLICY_READ_PROTECTIVE_MEASURES: &str = "read_protective_measures";
pub const POLICY_UPDATE_PROTECTIVE_MEASURES: &str = "update_protective_measures";
pub const POLICY_DELETE_PROTECTIVE_MEASURES: &str = "delete_protective_measures";

// Work Sessions policies
pub const POLICY_CREATE_WORK_SESSIONS: &str = "create_work_sessions";
pub const POLICY_UPDATE_WORK_SESSIONS: &str = "update_work_sessions";
pub const POLICY_END_WORK_SESSIONS: &str = "end_work_sessions";
pub const POLICY_VIEW_OTHER_WORK_SESSIONS: &str = "view_other_work_sessions";

// Attendance Members policies
pub const POLICY_MANAGE_ATTENDANCE_MEMBERS: &str = "manage_attendance_members";

// All valid policies array
pub const VALID_POLICIES: [&str; 29] = [
    POLICY_CREATE_CITIES,
    POLICY_READ_CITIES,
    POLICY_UPDATE_CITIES,
    POLICY_DELETE_CITIES,
    POLICY_CREATE_USERS,
    POLICY_READ_USERS,
    POLICY_UPDATE_USERS,
    POLICY_DELETE_USERS,
    POLICY_CREATE_VICTIMS,
    POLICY_READ_VICTIMS,
    POLICY_UPDATE_VICTIMS,
    POLICY_DELETE_VICTIMS,
    POLICY_CREATE_OFFENDERS,
    POLICY_READ_OFFENDERS,
    POLICY_UPDATE_OFFENDERS,
    POLICY_DELETE_OFFENDERS,
    POLICY_CREATE_ATTENDANCES,
    POLICY_READ_ATTENDANCES,
    POLICY_UPDATE_ATTENDANCES,
    POLICY_DELETE_ATTENDANCES,
    POLICY_CREATE_PROTECTIVE_MEASURES,
    POLICY_READ_PROTECTIVE_MEASURES,
    POLICY_UPDATE_PROTECTIVE_MEASURES,
    POLICY_DELETE_PROTECTIVE_MEASURES,
    POLICY_CREATE_WORK_SESSIONS,
    POLICY_UPDATE_WORK_SESSIONS,
    POLICY_END_WORK_SESSIONS,
    POLICY_VIEW_OTHER_WORK_SESSIONS,
    POLICY_MANAGE_ATTENDANCE_MEMBERS,
];

// Policies that are inherent to ROOT and must not be assignable
pub const NON_ASSIGNABLE_POLICIES: [&str; 3] = [
    POLICY_CREATE_CITIES,
    POLICY_UPDATE_CITIES,
    POLICY_DELETE_CITIES,
];

// Default CRUD policies for CITY_ADMIN
pub const CITY_ADMIN_DEFAULT_POLICIES: [&str; 26] = [
    POLICY_READ_CITIES,
    POLICY_CREATE_USERS,
    POLICY_READ_USERS,
    POLICY_UPDATE_USERS,
    POLICY_DELETE_USERS,
    POLICY_CREATE_VICTIMS,
    POLICY_READ_VICTIMS,
    POLICY_UPDATE_VICTIMS,
    POLICY_DELETE_VICTIMS,
    POLICY_CREATE_OFFENDERS,
    POLICY_READ_OFFENDERS,
    POLICY_UPDATE_OFFENDERS,
    POLICY_DELETE_OFFENDERS,
    POLICY_CREATE_ATTENDANCES,
    POLICY_READ_ATTENDANCES,
    POLICY_UPDATE_ATTENDANCES,
    POLICY_DELETE_ATTENDANCES,
    POLICY_CREATE_PROTECTIVE_MEASURES,
    POLICY_READ_PROTECTIVE_MEASURES,
    POLICY_UPDATE_PROTECTIVE_MEASURES,
    POLICY_DELETE_PROTECTIVE_MEASURES,
    POLICY_CREATE_WORK_SESSIONS,
    POLICY_UPDATE_WORK_SESSIONS,
    POLICY_END_WORK_SESSIONS,
    POLICY_VIEW_OTHER_WORK_SESSIONS,
    POLICY_MANAGE_ATTENDANCE_MEMBERS,
];

pub const CITY_USER_DEFAULT_POLICIES: [&str; 10] = [
    POLICY_READ_CITIES,
    POLICY_READ_VICTIMS,
    POLICY_READ_OFFENDERS,
    POLICY_READ_ATTENDANCES,
    POLICY_READ_PROTECTIVE_MEASURES,
    POLICY_READ_USERS,
    POLICY_CREATE_WORK_SESSIONS,
    POLICY_UPDATE_WORK_SESSIONS,
    POLICY_END_WORK_SESSIONS,
    POLICY_MANAGE_ATTENDANCE_MEMBERS,
];

pub fn is_valid_policy(policy: &str) -> bool {
    VALID_POLICIES.contains(&policy)
}

pub fn is_valid_profile(profile: &str) -> bool {
    VALID_PROFILES.contains(&profile)
}

pub fn is_assignable_policy(policy: &str) -> bool {
    !NON_ASSIGNABLE_POLICIES.contains(&policy)
}

pub fn generate_default_policies(
    profile: &str,
    city_id: Option<Uuid>,
) -> HashMap<String, Vec<Uuid>> {
    let mut policies: HashMap<String, Vec<Uuid>> = HashMap::new();

    match profile {
        PROFILE_ROOT => {
            // ROOT has implicit access to all policies, no need to store them
        }
        PROFILE_CITY_ADMIN => {
            if let Some(cid) = city_id {
                for policy in CITY_ADMIN_DEFAULT_POLICIES.iter() {
                    policies.insert(policy.to_string(), vec![cid]);
                }
            }
        }
        PROFILE_CITY_USER => {
            if let Some(cid) = city_id {
                for policy in CITY_USER_DEFAULT_POLICIES.iter() {
                    policies.insert(policy.to_string(), vec![cid]);
                }
            }
        }
        _ => {}
    }

    policies
}
