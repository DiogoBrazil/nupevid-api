use crate::core::entities::work_session_members::TeamMemberFunction;
use crate::core::value_objects::team_composition;
use uuid::Uuid;

pub struct WorkSessionValidator;

impl WorkSessionValidator {
    pub fn validate_team_functions(
        members: &[(Uuid, Option<TeamMemberFunction>)],
    ) -> Result<(), String> {
        team_composition::validate_team_functions(members)
    }

    pub fn can_add_member_with_function(
        current_members: &[(Uuid, Option<TeamMemberFunction>)],
        new_function: &Option<TeamMemberFunction>,
    ) -> Result<(), String> {
        team_composition::can_add_member_with_function(current_members, new_function)
    }

    pub fn can_remove_member(
        current_members: &[(Uuid, Option<TeamMemberFunction>)],
        member_to_remove: Uuid,
    ) -> Result<(), String> {
        team_composition::can_remove_member(current_members, member_to_remove)
    }
}
