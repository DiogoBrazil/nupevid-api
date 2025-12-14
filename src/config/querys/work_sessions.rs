pub struct WorkSessionsQueries;

impl WorkSessionsQueries {
    pub const CREATE_WORK_SESSION: &'static str = r#"
        INSERT INTO work_sessions (id, created_by_user_id, description)
        VALUES ($1, $2, $3)
        RETURNING id, created_by_user_id, started_at, ended_at, is_active,
                  description, created_at, updated_at
    "#;

    pub const GET_ACTIVE_SESSION_BY_USER: &'static str = r#"
        SELECT id, created_by_user_id, started_at, ended_at, is_active,
               description, created_at, updated_at
        FROM work_sessions
        WHERE created_by_user_id = $1 AND is_active = true
    "#;

    pub const GET_USER_ACTIVE_SESSION: &'static str = r#"
        SELECT ws.id, ws.created_by_user_id, ws.started_at, ws.ended_at,
               ws.is_active, ws.description, ws.created_at, ws.updated_at
        FROM work_sessions ws
        JOIN work_session_members wsm ON wsm.work_session_id = ws.id
        WHERE wsm.user_id = $1 AND ws.is_active = true
        LIMIT 1
    "#;

    pub const CHECK_USER_IN_ACTIVE_SESSION: &'static str = r#"
        SELECT EXISTS(
            SELECT 1
            FROM work_session_members wsm
            JOIN work_sessions ws ON ws.id = wsm.work_session_id
            WHERE wsm.user_id = $1 AND ws.is_active = true
        ) as in_active_session
    "#;

    pub const END_WORK_SESSION: &'static str = r#"
        UPDATE work_sessions
        SET is_active = false, ended_at = NOW()
        WHERE id = $1 AND is_active = true
        RETURNING id, created_by_user_id, started_at, ended_at, is_active,
                  description, created_at, updated_at
    "#;

    pub const GET_SESSION_BY_ID: &'static str = r#"
        SELECT id, created_by_user_id, started_at, ended_at, is_active,
               description, created_at, updated_at
        FROM work_sessions
        WHERE id = $1
    "#;

    pub const GET_SESSIONS_BY_USER: &'static str = r#"
        SELECT id, created_by_user_id, started_at, ended_at, is_active,
               description, created_at, updated_at
        FROM work_sessions
        WHERE created_by_user_id = $1
        ORDER BY started_at DESC
    "#;
}

pub struct WorkSessionMembersQueries;

impl WorkSessionMembersQueries {
    pub const CREATE_WORK_SESSION_MEMBER: &'static str = r#"
        INSERT INTO work_session_members (id, work_session_id, user_id, function)
        VALUES ($1, $2, $3, $4)
        RETURNING id, work_session_id, user_id, function, created_at
    "#;

    pub const GET_SESSION_MEMBERS: &'static str = r#"
        SELECT id, work_session_id, user_id, function, created_at
        FROM work_session_members
        WHERE work_session_id = $1
        ORDER BY created_at ASC
    "#;

    pub const GET_SESSION_MEMBERS_WITH_DETAILS: &'static str = r#"
        SELECT
            wsm.id,
            wsm.user_id,
            u.full_name as user_name,
            wsm.function,
            wsm.created_at
        FROM work_session_members wsm
        JOIN users u ON u.id = wsm.user_id
        WHERE wsm.work_session_id = $1
        ORDER BY wsm.created_at ASC
    "#;

    pub const DELETE_MEMBER: &'static str = r#"
        DELETE FROM work_session_members
        WHERE work_session_id = $1 AND user_id = $2
        RETURNING id, work_session_id, user_id, function, created_at
    "#;

    pub const DELETE_ALL_MEMBERS: &'static str = r#"
        DELETE FROM work_session_members
        WHERE work_session_id = $1
    "#;

    pub const UPDATE_MEMBER_FUNCTION: &'static str = r#"
        UPDATE work_session_members
        SET function = $3
        WHERE work_session_id = $1 AND user_id = $2
        RETURNING id, work_session_id, user_id, function, created_at
    "#;
}
