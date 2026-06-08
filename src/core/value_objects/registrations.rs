pub const REGISTRATION_PREFIX: &str = "1000";
pub const REGISTRATION_MAX_LENGTH: usize = 9;

pub fn is_valid_registration(registration: &str) -> bool {
    registration.len() <= REGISTRATION_MAX_LENGTH
        && registration.starts_with(REGISTRATION_PREFIX)
        && registration.chars().all(|c| c.is_ascii_digit())
}
