pub struct AttendanceVictimMembersQueries;

impl AttendanceVictimMembersQueries {
    pub const ADD_MEMBER: &'static str = r#"
        INSERT INTO attendance_victim_members (id, attendance_victim_id, user_id, work_session_id)
        VALUES ($1, $2, $3, $4)
        RETURNING id, attendance_victim_id, user_id, work_session_id, created_at
    "#;

    pub const GET_MEMBERS_WITH_DETAILS: &'static str = r#"
        SELECT
            avm.id,
            avm.user_id,
            u.full_name as user_name,
            avm.work_session_id,
            avm.created_at
        FROM attendance_victim_members avm
        JOIN users u ON u.id = avm.user_id
        WHERE avm.attendance_victim_id = $1
        ORDER BY avm.created_at ASC
    "#;

    pub const REMOVE_MEMBER: &'static str = r#"
        DELETE FROM attendance_victim_members
        WHERE attendance_victim_id = $1 AND user_id = $2
        RETURNING id, attendance_victim_id, user_id, work_session_id, created_at
    "#;
}

pub struct AttendanceOffenderMembersQueries;

impl AttendanceOffenderMembersQueries {
    pub const ADD_MEMBER: &'static str = r#"
        INSERT INTO attendance_offender_members (id, attendance_offender_id, user_id, work_session_id)
        VALUES ($1, $2, $3, $4)
        RETURNING id, attendance_offender_id, user_id, work_session_id, created_at
    "#;

    pub const GET_MEMBERS_WITH_DETAILS: &'static str = r#"
        SELECT
            aom.id,
            aom.user_id,
            u.full_name as user_name,
            aom.work_session_id,
            aom.created_at
        FROM attendance_offender_members aom
        JOIN users u ON u.id = aom.user_id
        WHERE aom.attendance_offender_id = $1
        ORDER BY aom.created_at ASC
    "#;

    pub const REMOVE_MEMBER: &'static str = r#"
        DELETE FROM attendance_offender_members
        WHERE attendance_offender_id = $1 AND user_id = $2
        RETURNING id, attendance_offender_id, user_id, work_session_id, created_at
    "#;
}
