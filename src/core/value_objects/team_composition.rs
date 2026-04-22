use crate::core::entities::work_session_members::TeamMemberFunction;
use uuid::Uuid;

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
                return Err(
                    "Team already has a Commander. Remove or change the current Commander first."
                        .to_string(),
                );
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

pub fn can_remove_member(
    current_members: &[(Uuid, Option<TeamMemberFunction>)],
    member_to_remove: Uuid,
) -> Result<(), String> {
    if current_members.len() <= 1 {
        return Err("Cannot remove member. Team must have at least one member.".to_string());
    }

    let is_commander = current_members.iter().any(|(id, func)| {
        *id == member_to_remove && matches!(func, Some(TeamMemberFunction::Commander))
    });

    if is_commander {
        return Err(
            "Cannot remove the Commander. Change the Commander role to another member first."
                .to_string(),
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_valid_commander_driver_patroller() {
        let members = vec![
            (Uuid::new_v4(), Some(TeamMemberFunction::Commander)),
            (Uuid::new_v4(), Some(TeamMemberFunction::Driver)),
            (Uuid::new_v4(), Some(TeamMemberFunction::Patroller)),
        ];
        assert!(validate_team_functions(&members).is_ok());
    }

    #[test]
    fn test_valid_commander_only() {
        let members = vec![(Uuid::new_v4(), Some(TeamMemberFunction::Commander))];
        assert!(validate_team_functions(&members).is_ok());
    }

    #[test]
    fn test_valid_commander_multiple_patrollers() {
        let members = vec![
            (Uuid::new_v4(), Some(TeamMemberFunction::Commander)),
            (Uuid::new_v4(), Some(TeamMemberFunction::Patroller)),
            (Uuid::new_v4(), Some(TeamMemberFunction::Patroller)),
            (Uuid::new_v4(), Some(TeamMemberFunction::Patroller)),
        ];
        assert!(validate_team_functions(&members).is_ok());
    }

    #[test]
    fn test_no_commander_err() {
        let members = vec![
            (Uuid::new_v4(), Some(TeamMemberFunction::Driver)),
            (Uuid::new_v4(), Some(TeamMemberFunction::Patroller)),
        ];
        let err = validate_team_functions(&members).unwrap_err();
        assert!(err.contains("exactly one Commander"));
    }

    #[test]
    fn test_two_commanders_err() {
        let members = vec![
            (Uuid::new_v4(), Some(TeamMemberFunction::Commander)),
            (Uuid::new_v4(), Some(TeamMemberFunction::Commander)),
        ];
        let err = validate_team_functions(&members).unwrap_err();
        assert!(err.contains("only have one Commander"));
    }

    #[test]
    fn test_two_drivers_err() {
        let members = vec![
            (Uuid::new_v4(), Some(TeamMemberFunction::Commander)),
            (Uuid::new_v4(), Some(TeamMemberFunction::Driver)),
            (Uuid::new_v4(), Some(TeamMemberFunction::Driver)),
        ];
        let err = validate_team_functions(&members).unwrap_err();
        assert!(err.contains("only have one Driver"));
    }

    #[test]
    fn test_valid_commander_driver_no_patrollers() {
        let members = vec![
            (Uuid::new_v4(), Some(TeamMemberFunction::Commander)),
            (Uuid::new_v4(), Some(TeamMemberFunction::Driver)),
        ];
        assert!(validate_team_functions(&members).is_ok());
    }

    #[test]
    fn test_valid_members_with_none_function() {
        let members = vec![
            (Uuid::new_v4(), Some(TeamMemberFunction::Commander)),
            (Uuid::new_v4(), None),
            (Uuid::new_v4(), None),
        ];
        assert!(validate_team_functions(&members).is_ok());
    }

    #[test]
    fn test_can_add_patroller() {
        let current = vec![(Uuid::new_v4(), Some(TeamMemberFunction::Commander))];
        assert!(
            can_add_member_with_function(&current, &Some(TeamMemberFunction::Patroller)).is_ok()
        );
    }

    #[test]
    fn test_can_add_driver_when_no_driver() {
        let current = vec![(Uuid::new_v4(), Some(TeamMemberFunction::Commander))];
        assert!(can_add_member_with_function(&current, &Some(TeamMemberFunction::Driver)).is_ok());
    }

    #[test]
    fn test_cannot_add_commander_when_exists() {
        let current = vec![(Uuid::new_v4(), Some(TeamMemberFunction::Commander))];
        let err = can_add_member_with_function(&current, &Some(TeamMemberFunction::Commander))
            .unwrap_err();
        assert!(err.contains("already has a Commander"));
    }

    #[test]
    fn test_cannot_add_driver_when_exists() {
        let current = vec![
            (Uuid::new_v4(), Some(TeamMemberFunction::Commander)),
            (Uuid::new_v4(), Some(TeamMemberFunction::Driver)),
        ];
        let err =
            can_add_member_with_function(&current, &Some(TeamMemberFunction::Driver)).unwrap_err();
        assert!(err.contains("already has a Driver"));
    }

    #[test]
    fn test_can_add_member_with_none() {
        let current = vec![(Uuid::new_v4(), Some(TeamMemberFunction::Commander))];
        assert!(can_add_member_with_function(&current, &None).is_ok());
    }

    #[test]
    fn test_can_add_commander_when_no_commander() {
        let current = vec![(Uuid::new_v4(), Some(TeamMemberFunction::Patroller))];
        assert!(
            can_add_member_with_function(&current, &Some(TeamMemberFunction::Commander)).is_ok()
        );
    }

    #[test]
    fn test_can_remove_patroller() {
        let pat_id = Uuid::new_v4();
        let current = vec![
            (Uuid::new_v4(), Some(TeamMemberFunction::Commander)),
            (pat_id, Some(TeamMemberFunction::Patroller)),
        ];
        assert!(can_remove_member(&current, pat_id).is_ok());
    }

    #[test]
    fn test_cannot_remove_only_member() {
        let cmd_id = Uuid::new_v4();
        let current = vec![(cmd_id, Some(TeamMemberFunction::Commander))];
        let err = can_remove_member(&current, cmd_id).unwrap_err();
        assert!(err.contains("at least one member"));
    }

    #[test]
    fn test_cannot_remove_commander() {
        let cmd_id = Uuid::new_v4();
        let current = vec![
            (cmd_id, Some(TeamMemberFunction::Commander)),
            (Uuid::new_v4(), Some(TeamMemberFunction::Patroller)),
        ];
        let err = can_remove_member(&current, cmd_id).unwrap_err();
        assert!(err.contains("Cannot remove the Commander"));
    }

    #[test]
    fn test_can_remove_driver() {
        let drv_id = Uuid::new_v4();
        let current = vec![
            (Uuid::new_v4(), Some(TeamMemberFunction::Commander)),
            (drv_id, Some(TeamMemberFunction::Driver)),
        ];
        assert!(can_remove_member(&current, drv_id).is_ok());
    }

    #[test]
    fn test_can_remove_none_function_member() {
        let none_id = Uuid::new_v4();
        let current = vec![
            (Uuid::new_v4(), Some(TeamMemberFunction::Commander)),
            (none_id, None),
        ];
        assert!(can_remove_member(&current, none_id).is_ok());
    }
}
