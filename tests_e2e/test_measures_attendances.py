"""Test protective measures, attendances and work sessions."""
from helpers import *


def _pm_payload(victim_id, offender_id, city_id, process_suffix, status="Valid", issued_at="2025-01-15"):
    return {
        "process_number": f"12345-{RUN_ID}.{process_suffix}.8.22.0001",
        "sei_process_number": f"SEI-{RUN_ID}/{process_suffix}",
        "occurrence_report_number": f"BO-{RUN_ID}/{process_suffix}",
        "issued_at": issued_at,
        "judicial_authority": "1a Vara",
        "court_district_id": city_id,
        "distance_meters": 500,
        "status": status,
        "violence_types": ["Physical", "Psychological"],
        "relationship_to_victim": "Spouse",
        "assaults_children": False,
        "was_drunk_during_assault": True,
        "victim_id": victim_id,
        "offender_id": offender_id,
    }


def _attendance_victim_payload(pm_id, city_id, risk_level="Medium", description="Atendimento vitima teste"):
    return {
        "protective_measure_id": pm_id,
        "was_victim_present": True,
        "attendance_date": "2025-03-15",
        "attendance_time": "14:30:00",
        "description": description,
        "latitude": -8.7612,
        "longitude": -63.9004,
        "address": {
            "street": "Rua das Flores",
            "number": "123",
            "district": "Centro",
            "city_id": city_id,
            "zip_code": "76801-000",
        },
        "is_remote": False,
        "risk_level": risk_level,
        "offender_freedom_status": "Free",
        "offender_has_firearm_access": "Unknown",
        "needs_legal_assistance": False,
        "needs_psychological_support": True,
        "was_instructed_about_protective_measure_procedures": True,
        "offender_violated_protective_measure": False,
    }


def _attendance_offender_payload(pm_id, city_id, aggravator="AlcoholUse", description="Atendimento agressor"):
    return {
        "protective_measure_id": pm_id,
        "was_offender_present": True,
        "attendance_date": "2025-03-15",
        "attendance_time": "14:30:00",
        "address": {
            "street": "Rua das Flores",
            "number": "123",
            "district": "Centro",
            "city_id": city_id,
            "zip_code": "76801-000",
        },
        "is_remote": False,
        "assaults_children": False,
        "violence_aggravator": aggravator,
        "description": description,
    }


def _ws_payload(description, members):
    return {"description": description, "members": members}


def test_protective_measures(root_token, admin_token, user_token, victim_id, offender_id, victim_b_id, offender_b_id, city_a, city_b):
    section("PROTECTIVE MEASURES")
    ctx = {"pm_ids": []}

    create_response = do(
        "POST",
        "protective-measures",
        root_token,
        json_data=_pm_payload(victim_id, offender_id, city_a, "001"),
        expected=201,
        label="create protective measure",
    )
    pm = data(create_response, "PM create data") or {}
    pm_id = pm.get("id")
    ctx["revoked_pm_id"] = pm_id
    ctx["pm_ids"].append(pm_id)
    require_id(pm_id, "PM id exists")
    expect_field(pm, "victim_id", victim_id, "PM victim_id")
    expect_field(pm, "offender_id", offender_id, "PM offender_id")
    expect_field(pm, "status", "Valid", "PM status")

    do(
        "POST",
        "protective-measures",
        root_token,
        json_data=_pm_payload(victim_id, offender_id, city_a, "002"),
        expected=409,
        label="create duplicate PM same active pair -> 409",
    )
    do(
        "POST",
        "protective-measures",
        root_token,
        json_data=_pm_payload(victim_id, offender_id, city_a, "003", issued_at="2999-01-01"),
        expected=400,
        label="create PM future issued_at -> 400",
    )
    invalid_required = _pm_payload(victim_id, offender_id, city_a, "004")
    invalid_required["judicial_authority"] = ""
    do("POST", "protective-measures", root_token, json_data=invalid_required, expected=400, label="create PM empty judicial_authority -> 400")
    invalid_status = _pm_payload(victim_id, offender_id, city_a, "005")
    invalid_status["status"] = "Expired"
    do("POST", "protective-measures", root_token, json_data=invalid_status, expected=422, label="create PM invalid status -> 422")
    do(
        "POST",
        "protective-measures",
        root_token,
        json_data=_pm_payload(UUID_ZERO, offender_id, city_a, "006"),
        expected=404,
        label="create PM victim not found -> 404",
    )

    list_response = do("GET", "protective-measures", root_token, expected=200, label="list protective measures")
    list_payload = expect_pagination(list_response, "PM pagination", min_items=1) or {}
    expect_contains_id(list_payload.get("data"), pm_id, "list PM contains created PM")

    if pm_id:
        detail = do("GET", f"protective-measures/{pm_id}", root_token, expected=200, label="get PM by id")
        expect_field(data(detail, "PM detail data") or {}, "id", pm_id, "PM detail id")

        related = do(
            "GET",
            f"protective-measures/{pm_id}",
            root_token,
            params={"include_related_entities": "true"},
            expected=200,
            label="get PM with related entities",
        )
        related_data = data(related, "PM related data") or {}
        expect_truth("victim" in related_data, "PM related includes victim")
        expect_truth("offender" in related_data, "PM related includes offender")
        expect_truth("court_district" in related_data, "PM related includes court_district")

        by_victim = do("GET", f"protective-measures/victim/{victim_id}", root_token, expected=200, label="get PMs by victim")
        expect_contains_id(data(by_victim, "PMs by victim data"), pm_id, "PMs by victim contains created PM")

        update_payload = _pm_payload(victim_id, offender_id, city_a, "001")
        update_payload["distance_meters"] = 300
        update_payload["violence_types"] = ["Physical", "Psychological", "Moral"]
        update_response = do("PUT", f"protective-measures/{pm_id}", root_token, json_data=update_payload, expected=200, label="update PM")
        expect_field(data(update_response, "PM update data") or {}, "distance_meters", 300, "PM distance updated")

        revoke_payload = _pm_payload(victim_id, offender_id, city_a, "001", status="Revoked")
        revoke_response = do("PUT", f"protective-measures/{pm_id}", root_token, json_data=revoke_payload, expected=200, label="revoke PM")
        expect_field(data(revoke_response, "PM revoke data") or {}, "status", "Revoked", "PM revoked")

    new_pm_response = do(
        "POST",
        "protective-measures",
        root_token,
        json_data=_pm_payload(victim_id, offender_id, city_a, "007"),
        expected=201,
        label="create new active PM after revocation",
    )
    new_pm = data(new_pm_response, "new active PM data") or {}
    active_pm_id = new_pm.get("id")
    ctx["pm_id"] = active_pm_id
    ctx["pm_ids"].append(active_pm_id)
    require_id(active_pm_id, "new active PM id exists")
    expect_not_equal(active_pm_id, pm_id, "new active PM differs from revoked PM")

    do(
        "POST",
        "protective-measures",
        user_token,
        json_data=_pm_payload(victim_id, offender_id, city_a, "008"),
        expected=403,
        label="CITY_USER create PM without policy -> 403",
    )
    do(
        "POST",
        "protective-measures",
        admin_token,
        json_data=_pm_payload(victim_b_id, offender_b_id, city_b, "009"),
        expected=403,
        label="CITY_ADMIN create PM other city -> 403",
    )
    do("GET", "protective-measures", admin_token, expected=200, label="CITY_ADMIN list PMs -> 200")
    do("GET", "protective-measures", user_token, expected=200, label="CITY_USER list PMs -> 200")

    do("GET", f"protective-measures/{UUID_ZERO}", root_token, expected=404, label="get PM not found -> 404")

    return ctx


def test_work_sessions(root_token, root_id, users, admin_token, admin_email, city_a, city_b, pm_id):
    section("WORK SESSIONS")

    admin_id = users["admin_id"]
    user_b_id = users["user_b_id"]
    extra_user_id = users.get("extra_user_id")

    do("POST", "work-sessions/end", root_token, expected=(200, 404), label="end any active root session")

    # Attendances must fail before there is an active work session.
    do(
        "POST",
        "attendance-victims",
        root_token,
        json_data=_attendance_victim_payload(pm_id, city_a),
        expected=400,
        label="create attendance victim without active session -> 400",
    )

    root_member = {"user_id": root_id, "function": "Commander"}
    response = do(
        "POST",
        "work-sessions",
        root_token,
        json_data=_ws_payload("Patrulhamento teste", [root_member]),
        expected=201,
        label="ROOT create work session",
    )
    ws = data(response, "ROOT work session data") or {}
    ws_id = ws.get("id")
    require_id(ws_id, "ROOT work session id exists")
    expect_field(ws, "is_active", True, "ROOT work session active")

    do(
        "POST",
        "work-sessions",
        root_token,
        json_data=_ws_payload("Duplicate active", [root_member]),
        expected=409,
        label="create second active ROOT session -> 409",
    )

    active = do("GET", "work-sessions/active", root_token, expected=200, label="get active session")
    expect_field(data(active, "active session data") or {}, "id", ws_id, "active session id")

    detail = do("GET", f"work-sessions/{ws_id}", root_token, expected=200, label="get work session by id")
    expect_contains((data(detail, "work session detail data") or {}).get("members"), lambda m: m.get("user_id") == root_id, "work session detail contains root member")

    list_response = do(
        "GET",
        "work-sessions",
        root_token,
        params={"include_related_entities": "true", "page": 1, "page_size": 20},
        expected=200,
        label="list work sessions with related entities",
    )
    expect_pagination(list_response, "work sessions pagination", min_items=1)
    list_by_user = do(
        "GET",
        "work-sessions",
        root_token,
        params={"user_id": root_id, "page": 1, "page_size": 20},
        expected=200,
        label="list work sessions filtered by user",
    )
    by_user_payload = expect_pagination(list_by_user, "work sessions by user pagination", min_items=1) or {}
    expect_contains_id(by_user_payload.get("data"), ws_id, "work sessions by user contains root session")

    update_response = do(
        "PUT",
        f"work-sessions/{ws_id}",
        root_token,
        json_data=_ws_payload("Patrulhamento atualizado", [root_member]),
        expected=200,
        label="update work session",
    )
    expect_field(data(update_response, "work session update data") or {}, "description", "Patrulhamento atualizado", "work session description updated")

    do(
        "POST",
        "work-sessions",
        root_token,
        json_data=_ws_payload("No Commander", [{"user_id": root_id, "function": "Patroller"}]),
        expected=400,
        label="create session no Commander -> 400",
    )
    do(
        "POST",
        "work-sessions",
        root_token,
        json_data=_ws_payload("Two Commanders", [root_member, {"user_id": admin_id, "function": "Commander"}]),
        expected=400,
        label="create session two Commanders -> 400",
    )
    if extra_user_id:
        do(
            "POST",
            "work-sessions",
            root_token,
            json_data=_ws_payload(
                "Two Drivers",
                [root_member, {"user_id": admin_id, "function": "Driver"}, {"user_id": extra_user_id, "function": "Driver"}],
            ),
            expected=400,
            label="create session two Drivers -> 400",
        )

    do("POST", "work-sessions/end", root_token, expected=200, label="end work session")
    do("GET", "work-sessions/active", root_token, expected=404, label="get active session after end -> 404")

    # CITY_ADMIN flow.
    admin_token2, _, _ = login(admin_email, "senha@123", expected=200)
    if admin_token2:
        do("POST", "work-sessions/end", admin_token2, expected=(200, 404), label="end any active admin session")
        admin_session = do(
            "POST",
            "work-sessions",
            admin_token2,
            json_data=_ws_payload("Patrulha Admin", [{"user_id": admin_id, "function": "Commander"}]),
            expected=201,
            label="CITY_ADMIN create work session -> 201",
        )
        admin_ws_id = (data(admin_session, "admin work session data") or {}).get("id")
        require_id(admin_ws_id, "CITY_ADMIN work session id exists")
        do(
            "POST",
            "work-sessions",
            admin_token2,
            json_data=_ws_payload(
                "Admin Cross City",
                [{"user_id": admin_id, "function": "Commander"}, {"user_id": user_b_id, "function": "Patroller"}],
            ),
            expected=400,
            label="CITY_ADMIN create session with member from other city -> 400",
        )
        do("POST", "work-sessions/end", admin_token2, expected=200, label="CITY_ADMIN end session")

    # Final active session required for attendance tests.
    final_response = do(
        "POST",
        "work-sessions",
        root_token,
        json_data=_ws_payload("Session for attendances", [root_member]),
        expected=201,
        label="create session for attendance tests",
    )
    final_ws = data(final_response, "final work session data") or {}
    final_ws_id = final_ws.get("id")
    require_id(final_ws_id, "final active work session id exists")

    if final_ws_id:
        do("DELETE", f"work-sessions/{final_ws_id}/members/{root_id}", root_token, expected=400, label="remove Commander from WS -> 400")
        do(
            "POST",
            f"work-sessions/{final_ws_id}/members",
            root_token,
            json_data={"user_id": admin_id, "function": "Patroller"},
            expected=200,
            label="add member to WS",
        )
        do(
            "POST",
            f"work-sessions/{final_ws_id}/members",
            root_token,
            json_data={"user_id": user_b_id, "function": "Patroller"},
            expected=400,
            label="add member from other city to WS -> 400",
        )
        if extra_user_id:
            do(
                "POST",
                f"work-sessions/{final_ws_id}/members",
                root_token,
                json_data={"user_id": extra_user_id, "function": "Driver"},
                expected=200,
                label="add driver to WS",
            )
            do(
                "PUT",
                f"work-sessions/{final_ws_id}/members/{admin_id}/function",
                root_token,
                json_data={"function": "Driver"},
                expected=400,
                label="update WS member to duplicate Driver -> 400",
            )
            do(
                "PUT",
                f"work-sessions/{final_ws_id}/members/{extra_user_id}/function",
                root_token,
                json_data={"function": "Patroller"},
                expected=200,
                label="update WS driver to patroller",
            )
        do(
            "PUT",
            f"work-sessions/{final_ws_id}/members/{admin_id}/function",
            root_token,
            json_data={"function": "Driver"},
            expected=200,
            label="update WS member function",
        )
        if extra_user_id:
            do(
                "PUT",
                f"work-sessions/{final_ws_id}/members",
                root_token,
                json_data={
                    "members": [
                        root_member,
                        {"user_id": admin_id, "function": "Driver"},
                        {"user_id": extra_user_id, "function": "Driver"},
                    ]
                },
                expected=400,
                label="update WS members with duplicate Driver -> 400",
            )
            do(
                "PUT",
                f"work-sessions/{final_ws_id}/members",
                root_token,
                json_data={
                    "members": [
                        root_member,
                        {"user_id": admin_id, "function": "Driver"},
                        {"user_id": extra_user_id, "function": "Patroller"},
                    ]
                },
                expected=200,
                label="update WS members valid set",
            )
            do("DELETE", f"work-sessions/{final_ws_id}/members/{extra_user_id}", root_token, expected=200, label="remove extra member from WS")
        do("DELETE", f"work-sessions/{final_ws_id}/members/{admin_id}", root_token, expected=200, label="remove member from WS")

    return final_ws_id


def test_attendance_victims(root_token, user_token, pm_id, city_a, admin_id, user_b_id, victim_id, root_id):
    section("ATTENDANCE VICTIMS")

    response = do(
        "POST",
        "attendance-victims",
        root_token,
        json_data=_attendance_victim_payload(pm_id, city_a),
        expected=201,
        label="create attendance victim",
    )
    attendance = data(response, "attendance victim create data") or {}
    av_id = attendance.get("id")
    require_id(av_id, "attendance victim id exists")
    expect_field(attendance, "protective_measure_id", pm_id, "attendance victim PM id")
    expect_field(attendance, "victim_id", victim_id, "attendance victim linked victim")
    expect_field(attendance, "risk_level", "Medium", "attendance victim risk level")

    do("POST", "attendance-victims", root_token, json_data={"was_victim_present": True, "attendance_date": "2025-03-15", "attendance_time": "14:30:00", "is_remote": False}, expected=422, label="create attendance victim no PM -> 422")
    do("POST", "attendance-victims", root_token, json_data=_attendance_victim_payload(UUID_ZERO, city_a), expected=404, label="create attendance victim PM not found -> 404")
    invalid_enum = _attendance_victim_payload(pm_id, city_a)
    invalid_enum["risk_level"] = "Extreme"
    do("POST", "attendance-victims", root_token, json_data=invalid_enum, expected=422, label="create attendance victim invalid enum -> 422")

    list_response = do("GET", "attendance-victims", root_token, expected=200, label="list attendance victims")
    list_payload = expect_pagination(list_response, "attendance victims pagination", min_items=1) or {}
    expect_contains_id(list_payload.get("data"), av_id, "list attendance victims contains created attendance")

    if av_id:
        detail = do("GET", f"attendance-victims/{av_id}", root_token, expected=200, label="get attendance victim by id")
        expect_field(data(detail, "attendance victim detail data") or {}, "id", av_id, "attendance victim detail id")

        update_payload = _attendance_victim_payload(pm_id, city_a, risk_level="High", description="Atendimento vitima atualizado")
        update_payload["attendance_time"] = "15:00:00"
        update_payload["offender_has_firearm_access"] = "No"
        update_payload["needs_legal_assistance"] = True
        update_response = do("PUT", f"attendance-victims/{av_id}", root_token, json_data=update_payload, expected=200, label="update attendance victim")
        expect_field(data(update_response, "attendance victim update data") or {}, "risk_level", "High", "attendance victim risk updated")

        by_measure = do("GET", f"attendance-victims/by-measure/{pm_id}", root_token, expected=200, label="get attendance victims by measure")
        expect_contains_id(data(by_measure, "attendance victims by measure data"), av_id, "attendance victims by measure contains created attendance")
        by_victim = do("GET", f"attendance-victims/by-victim/{victim_id}", root_token, expected=200, label="get attendance victims by victim")
        expect_contains_id(data(by_victim, "attendance victims by victim data"), av_id, "attendance victims by victim contains created attendance")

        members = do("GET", f"attendance-victims/{av_id}/members", root_token, expected=200, label="list attendance victim members")
        expect_contains(data(members, "attendance victim members data"), lambda m: m.get("user_id") == root_id, "attendance victim members include active session root")
        do("POST", f"attendance-victims/{av_id}/members", root_token, json_data={"user_id": UUID_ZERO}, expected=404, label="add missing member to attendance victim -> 404")
        do("POST", f"attendance-victims/{av_id}/members", root_token, json_data={"user_id": user_b_id}, expected=400, label="add member from other city to attendance victim -> 400")
        do("POST", f"attendance-victims/{av_id}/members", root_token, json_data={"user_id": admin_id}, expected=200, label="add member to attendance victim")
        members = do("GET", f"attendance-victims/{av_id}/members", root_token, expected=200, label="list attendance victim members after add")
        expect_contains(data(members, "attendance victim members after add data"), lambda m: m.get("user_id") == admin_id, "attendance victim members include admin")
        do("DELETE", f"attendance-victims/{av_id}/members/{admin_id}", root_token, expected=200, label="remove member from attendance victim")

        do("PUT", f"attendance-victims/{av_id}", user_token, json_data=update_payload, expected=403, label="CITY_USER update attendance victim without policy -> 403")
        do("DELETE", f"attendance-victims/{av_id}", user_token, expected=403, label="CITY_USER delete attendance victim without policy -> 403")

    do("GET", f"attendance-victims/{UUID_ZERO}", root_token, expected=404, label="get attendance victim not found -> 404")

    return av_id


def test_attendance_offenders(root_token, user_token, pm_id, city_a, admin_id, user_b_id, offender_id, victim_id, root_id):
    section("ATTENDANCE OFFENDERS")

    response = do(
        "POST",
        "attendance-offenders",
        root_token,
        json_data=_attendance_offender_payload(pm_id, city_a),
        expected=201,
        label="create attendance offender",
    )
    attendance = data(response, "attendance offender create data") or {}
    ao_id = attendance.get("id")
    require_id(ao_id, "attendance offender id exists")
    expect_field(attendance, "protective_measure_id", pm_id, "attendance offender PM id")
    expect_field(attendance, "offender_id", offender_id, "attendance offender linked offender")
    expect_field(attendance, "victim_id", victim_id, "attendance offender linked victim")

    do("POST", "attendance-offenders", root_token, json_data=_attendance_offender_payload(UUID_ZERO, city_a), expected=404, label="create attendance offender PM not found -> 404")
    invalid_enum = _attendance_offender_payload(pm_id, city_a)
    invalid_enum["violence_aggravator"] = "Invalid"
    do("POST", "attendance-offenders", root_token, json_data=invalid_enum, expected=422, label="create attendance offender invalid enum -> 422")

    list_response = do("GET", "attendance-offenders", root_token, expected=200, label="list attendance offenders")
    list_payload = expect_pagination(list_response, "attendance offenders pagination", min_items=1) or {}
    expect_contains_id(list_payload.get("data"), ao_id, "list attendance offenders contains created attendance")

    if ao_id:
        detail = do("GET", f"attendance-offenders/{ao_id}", root_token, expected=200, label="get attendance offender by id")
        expect_field(data(detail, "attendance offender detail data") or {}, "id", ao_id, "attendance offender detail id")

        update_payload = _attendance_offender_payload(pm_id, city_a, aggravator="DrugUse", description="Acompanhamento atualizado")
        update_response = do("PUT", f"attendance-offenders/{ao_id}", root_token, json_data=update_payload, expected=200, label="update attendance offender")
        expect_field(data(update_response, "attendance offender update data") or {}, "violence_aggravator", "DrugUse", "attendance offender aggravator updated")

        by_measure = do("GET", f"attendance-offenders/by-measure/{pm_id}", root_token, expected=200, label="get attendance offenders by measure")
        expect_contains_id(data(by_measure, "attendance offenders by measure data"), ao_id, "attendance offenders by measure contains created attendance")
        by_offender = do("GET", f"attendance-offenders/by-offender/{offender_id}", root_token, expected=200, label="get attendance offenders by offender")
        expect_contains_id(data(by_offender, "attendance offenders by offender data"), ao_id, "attendance offenders by offender contains created attendance")
        by_victim = do("GET", f"attendance-offenders/by-victim/{victim_id}", root_token, expected=200, label="get attendance offenders by victim")
        expect_contains_id(data(by_victim, "attendance offenders by victim data"), ao_id, "attendance offenders by victim contains created attendance")

        members = do("GET", f"attendance-offenders/{ao_id}/members", root_token, expected=200, label="list attendance offender members")
        expect_contains(data(members, "attendance offender members data"), lambda m: m.get("user_id") == root_id, "attendance offender members include active session root")
        do("POST", f"attendance-offenders/{ao_id}/members", root_token, json_data={"user_id": UUID_ZERO}, expected=404, label="add missing member to attendance offender -> 404")
        do("POST", f"attendance-offenders/{ao_id}/members", root_token, json_data={"user_id": user_b_id}, expected=400, label="add member from other city to attendance offender -> 400")
        do("POST", f"attendance-offenders/{ao_id}/members", root_token, json_data={"user_id": admin_id}, expected=200, label="add member to attendance offender")
        members = do("GET", f"attendance-offenders/{ao_id}/members", root_token, expected=200, label="list attendance offender members after add")
        expect_contains(data(members, "attendance offender members after add data"), lambda m: m.get("user_id") == admin_id, "attendance offender members include admin")
        do("DELETE", f"attendance-offenders/{ao_id}/members/{admin_id}", root_token, expected=200, label="remove member from attendance offender")

        do("PUT", f"attendance-offenders/{ao_id}", user_token, json_data=update_payload, expected=403, label="CITY_USER update attendance offender without policy -> 403")
        do("DELETE", f"attendance-offenders/{ao_id}", user_token, expected=403, label="CITY_USER delete attendance offender without policy -> 403")

    return ao_id


def test_attendance_permissions(user_token, pm_id):
    section("ATTENDANCE PERMISSIONS")

    do(
        "POST",
        "attendance-victims",
        user_token,
        json_data=_attendance_victim_payload(pm_id, None),
        expected=403,
        label="CITY_USER create attendance victim no policy -> 403",
    )
    do(
        "POST",
        "attendance-offenders",
        user_token,
        json_data=_attendance_offender_payload(pm_id, None),
        expected=403,
        label="CITY_USER create attendance offender no policy -> 403",
    )
    do("GET", "attendance-victims", user_token, expected=200, label="CITY_USER read attendance victims -> 200")
    do("GET", "attendance-offenders", user_token, expected=200, label="CITY_USER read attendance offenders -> 200")


def _unique(values):
    seen = set()
    out = []
    for value in values:
        if value and value not in seen:
            seen.add(value)
            out.append(value)
    return out


def test_cleanup_delete(root_token, av_id, ao_id, pm_ids, victim_ids, offender_ids, city_a, city_b, user_ids):
    section("DELETE / CLEANUP (soft delete)")

    if av_id:
        do("DELETE", f"attendance-victims/{av_id}", root_token, expected=200, label="delete attendance victim")
        do("GET", f"attendance-victims/{av_id}", root_token, expected=404, label="get deleted attendance victim -> 404")
    if ao_id:
        do("DELETE", f"attendance-offenders/{ao_id}", root_token, expected=200, label="delete attendance offender")
        do("GET", f"attendance-offenders/{ao_id}", root_token, expected=404, label="get deleted attendance offender -> 404")

    do("POST", "work-sessions/end", root_token, expected=(200, 404), label="end work session before cleanup")

    for pm_id in _unique(pm_ids):
        do("DELETE", f"protective-measures/{pm_id}", root_token, expected=200, label=f"delete protective measure {pm_id[:8]}")
        do("GET", f"protective-measures/{pm_id}", root_token, expected=404, label=f"get deleted PM {pm_id[:8]} -> 404")

    unique_victims = _unique(victim_ids)
    for victim_id in unique_victims:
        do("DELETE", f"victims/{victim_id}", root_token, expected=200, label=f"delete victim {victim_id[:8]}")
        do("GET", f"victims/{victim_id}", root_token, expected=404, label=f"get deleted victim {victim_id[:8]} -> 404")

    for offender_id in _unique(offender_ids):
        do("DELETE", f"offenders/{offender_id}", root_token, expected=200, label=f"delete offender {offender_id[:8]}")
        do("GET", f"offenders/{offender_id}", root_token, expected=404, label=f"get deleted offender {offender_id[:8]} -> 404")

    for user_id in _unique(user_ids):
        do("DELETE", f"users/{user_id}", root_token, expected=200, label=f"delete user {user_id[:8]}")
        do("GET", f"users/{user_id}", root_token, expected=404, label=f"get deleted user {user_id[:8]} -> 404")

    if city_a:
        do("DELETE", f"cities/{city_a}", root_token, expected=200, label="delete city A")
        do("GET", f"cities/{city_a}", root_token, expected=404, label="get deleted city A -> 404")
    if city_b:
        do("DELETE", f"cities/{city_b}", root_token, expected=200, label="delete city B")
        do("GET", f"cities/{city_b}", root_token, expected=404, label="get deleted city B -> 404")

    if unique_victims:
        do("DELETE", f"victims/{unique_victims[0]}", root_token, expected=404, label="delete victim again -> 404")
