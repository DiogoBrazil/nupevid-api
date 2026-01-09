use crate::core::entities::work_session_members::TeamMemberFunction;
use uuid::Uuid;

pub struct WorkSessionValidator;

impl WorkSessionValidator {
    pub fn validate_team_functions(
        members: &[(Uuid, Option<TeamMemberFunction>)],
    ) -> Result<(), String> {
        let mut commander_count = 0;
        let mut driver_count = 0;

        for (_, function) in members {
            match function {
                Some(TeamMemberFunction::Commander) => commander_count += 1,
                Some(TeamMemberFunction::Driver) => driver_count += 1,
                Some(TeamMemberFunction::Patroller) => {}
                None => {}
            }
        }

        if commander_count == 0 {
            return Err("Team must have exactly one Commander".to_string());
        }

        if commander_count > 1 {
            return Err("Team can only have one Commander".to_string());
        }

        if driver_count > 1 {
            return Err("Team can only have one Driver".to_string());
        }

        Ok(())
    }

    pub fn can_add_member_with_function(
        current_members: &[(Uuid, Option<TeamMemberFunction>)],
        new_function: &Option<TeamMemberFunction>,
    ) -> Result<(), String> {
        match new_function {
            Some(TeamMemberFunction::Commander) => {
                let has_commander = current_members
                    .iter()
                    .any(|(_, f)| matches!(f, Some(TeamMemberFunction::Commander)));

                if has_commander {
                    return Err("Team already has a Commander. Remove or change the current Commander first.".to_string());
                }
            }
            Some(TeamMemberFunction::Driver) => {
                let has_driver = current_members
                    .iter()
                    .any(|(_, f)| matches!(f, Some(TeamMemberFunction::Driver)));

                if has_driver {
                    return Err(
                        "Team already has a Driver. Remove or change the current Driver first."
                            .to_string(),
                    );
                }
            }
            _ => {}
        }

        Ok(())
    }

    pub fn can_remove_member(current_member_count: usize) -> Result<(), String> {
        if current_member_count <= 1 {
            return Err("Cannot remove member. Team must have at least one member.".to_string());
        }

        Ok(())
    }
}
