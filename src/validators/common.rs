use crate::utils::errors::AppError;
use regex::Regex;
use lazy_static::lazy_static;

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

// Default CRUD policies for CITY_ADMIN (all operations on their city, except city management)
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

pub const VALID_CITIES: [&str; 52] = [
    "ALTA FLORESTA D'OESTE",
    "ALTO ALEGRE DOS PARECIS",
    "ALTO PARAÍSO",
    "ALVORADA D'OESTE",
    "ARIQUEMES",
    "BURITIS",
    "CABIXI",
    "CACAULÂNDIA",
    "CACOAL",
    "CAMPO NOVO DE RONDÔNIA",
    "CANDEIAS DO JAMARI",
    "CASTANHEIRAS",
    "CEREJEIRAS",
    "CHUPINGUAIA",
    "COLORADO DO OESTE",
    "CORUMBIARA",
    "COSTA MARQUES",
    "CUJUBIM",
    "ESPIGÃO D'OESTE",
    "GOVERNADOR JORGE TEIXEIRA",
    "GUAJARÁ-MIRIM",
    "ITAPUÃ DO OESTE",
    "JARU",
    "JI-PARANÁ",
    "MACHADINHO D'OESTE",
    "MINISTRO ANDREAZZA",
    "MIRANTE DA SERRA",
    "MONTE NEGRO",
    "NOVA BRASILÂNDIA D'OESTE",
    "NOVA MAMORÉ",
    "NOVA UNIÃO",
    "NOVO HORIZONTE DO OESTE",
    "OURO PRETO DO OESTE",
    "PARECIS",
    "PIMENTA BUENO",
    "PIMENTEIRAS DO OESTE",
    "PORTO VELHO",
    "PRESIDENTE MÉDICI",
    "PRIMAVERA DE RONDÔNIA",
    "RIO CRESPO",
    "ROLIM DE MOURA",
    "SANTA LUZIA D'OESTE",
    "SÃO FELIPE D'OESTE",
    "SÃO FRANCISCO DO GUAPORÉ",
    "SÃO MIGUEL DO GUAPORÉ",
    "SERINGUEIRAS",
    "TEIXEIRÓPOLIS",
    "THEOBROMA",
    "URUPÁ",
    "VALE DO ANARI",
    "VALE DO PARAÍSO",
    "VILHENA",
];

// Valid battalions
pub const VALID_BATTALIONS: [&str; 16] = [
    "1ºBPM",
    "2ºBPM",
    "3ºBPM",
    "4ºBPM",
    "5ºBPM",
    "6ºBPM",
    "7ºBPM",
    "8ºBPM",
    "9ºBPM",
    "10ºBPM",
    "11ºBPM",
    "BPTRAN",
    "BOPE",
    "BPCHOQUE",
    "BPTAR",
    "CIPO/BURITIS",
];

pub const VALID_STATES: [&str; 1] = ["RO"];

// Registration prefix and max length
pub const REGISTRATION_PREFIX: &str = "1000";
pub const REGISTRATION_MAX_LENGTH: usize = 9;

pub const VALID_RANKS: [&str; 14] = [
    "CEL PM",
    "TC PM",
    "MAJ PM",
    "CAP PM",
    "1º TEN PM",
    "2º TEN PM",
    "ASP OF PM",
    "CAD PM",
    "ST PM",
    "1º SGT PM",
    "2º SGT PM",
    "3º SGT PM",
    "CB PM",
    "SD PM",
];

lazy_static!{
    static ref EMAIL_VALIDATION_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
}

pub fn is_valid_email(email: &str) -> bool {
    EMAIL_VALIDATION_REGEX.is_match(email)
}

pub fn is_valid_profile(profile: &str) -> bool {
    VALID_PROFILES.contains(&profile)
}

pub fn is_valid_city_name(name: &str) -> bool {
    VALID_CITIES.contains(&name.to_uppercase().as_str())
}

pub fn is_valid_battalion(battalion: &str) -> bool {
    VALID_BATTALIONS.contains(&battalion)
}

pub fn is_valid_state(state: &str) -> bool {
    VALID_STATES.contains(&state.to_uppercase().as_str())
}

pub fn is_valid_rank(rank: &str) -> bool {
    VALID_RANKS.contains(&rank)
}

pub fn is_valid_registration(registration: &str) -> bool {
    registration.len() <= REGISTRATION_MAX_LENGTH
        && registration.starts_with(REGISTRATION_PREFIX)
        && registration.chars().all(|c| c.is_ascii_digit())
}

pub fn is_public_route(path: &str) -> bool {
    let public_routes = [
        "/api/v1/auth/login",
        "/api/swagger",
    ];
    public_routes.iter().any(|route | path.starts_with(route))
}

pub fn validate_required_fields(validations: &[(&str, bool)], error_prefix: &str) -> Result<(), AppError> {
    for (field_name, is_empty) in validations {
        if *is_empty {
            return Err(AppError::BadRequest(
                format!("{}: {} cannot be empty", error_prefix, field_name)
            ));
        }
    }
    Ok(())
}

pub fn is_valid_policy(policy: &str) -> bool {
    VALID_POLICIES.contains(&policy)
}

pub fn is_assignable_policy(policy: &str) -> bool {
    !NON_ASSIGNABLE_POLICIES.contains(&policy)
}

pub fn generate_default_policies(profile: &str, city_id: Option<uuid::Uuid>) -> std::collections::HashMap<String, Vec<uuid::Uuid>> {
    use std::collections::HashMap;

    let mut policies: HashMap<String, Vec<uuid::Uuid>> = HashMap::new();

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
