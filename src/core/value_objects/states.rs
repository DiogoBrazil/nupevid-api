pub const VALID_STATES: [&str; 1] = ["RO"];

pub fn is_valid_state(state: &str) -> bool {
    VALID_STATES.contains(&state.to_uppercase().as_str())
}
