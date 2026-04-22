#!/usr/bin/env python3
"""
NUPEVID API - E2E Test Runner

Prerequisites:
  - API running at localhost:8080
  - pip install requests

Usage:
  python3 run_tests.py
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from helpers import RUN_ID, login, stats, summary
from test_auth_cities import test_auth, test_cities, test_cities_permission
from test_measures_attendances import (
    test_attendance_offenders,
    test_attendance_permissions,
    test_attendance_victims,
    test_cleanup_delete,
    test_protective_measures,
    test_work_sessions,
)
from test_users import test_password, test_users_crud, test_users_permissions
from test_victims_offenders import (
    test_offenders_by_victim,
    test_offenders_crud,
    test_victims_crud,
    test_victims_offenders_permissions,
)


def require_or_exit(value, message):
    if value:
        return value
    stats()["failed"] += 1
    stats()["errors"].append(f"❌ CRITICAL: {message}")
    print(f"\n💀 {message}")
    summary()
    sys.exit(1)


def main():
    print(f"🚀 NUPEVID API E2E Tests (run_id: {RUN_ID})")
    print("=" * 60)

    root_token, root_id = test_auth()
    require_or_exit(root_token and root_id, "Cannot proceed without ROOT token/id")

    city_a, city_b = test_cities(root_token)
    require_or_exit(city_a and city_b, "Cannot proceed without both cities")

    users = test_users_crud(root_token, city_a, city_b)
    require_or_exit(users.get("admin_id") and users.get("user_id") and users.get("user_b_id"), "Cannot proceed without baseline users")

    admin_token, _, _ = login(users["admin_email"], "senha@123", expected=200)
    user_token, _, _ = login(users["user_email"], "senha@123", expected=200)
    require_or_exit(admin_token and user_token, "Cannot login as CITY_ADMIN/CITY_USER")

    extra_user_id = test_users_permissions(root_token, admin_token, user_token, users, city_a, city_b)
    users["extra_user_id"] = extra_user_id

    # Re-login user to refresh policies added during permission tests.
    user_token, _, _ = login(users["user_email"], "senha@123", expected=200)
    require_or_exit(user_token, "Cannot re-login CITY_USER after policy changes")

    test_cities_permission(admin_token, user_token, city_a, city_b)

    user_pw = test_password(admin_token, user_token, users)
    user_token, _, _ = login(users["user_email"], user_pw, expected=200)
    require_or_exit(user_token, "Cannot re-login CITY_USER after password flow")

    victims = test_victims_crud(root_token, city_a, city_b)
    require_or_exit(victims.get("victim_id") and victims.get("victim_b_id"), "Cannot proceed without baseline victims")

    offenders = test_offenders_crud(root_token, city_a, city_b)
    require_or_exit(offenders.get("offender_id") and offenders.get("offender_b_id"), "Cannot proceed without baseline offenders")

    permission_entities = test_victims_offenders_permissions(
        admin_token,
        user_token,
        victims["victim_id"],
        offenders["offender_id"],
        city_a,
        city_b,
    )

    measures = test_protective_measures(
        root_token,
        admin_token,
        user_token,
        victims["victim_id"],
        offenders["offender_id"],
        victims["victim_b_id"],
        offenders["offender_b_id"],
        city_a,
        city_b,
    )
    pm_id = measures.get("pm_id")
    require_or_exit(pm_id, "Cannot proceed without active protective measure")

    test_offenders_by_victim(root_token, victims["victim_id"], offenders["offender_id"])

    test_work_sessions(root_token, root_id, users, admin_token, users["admin_email"], city_a, city_b, pm_id)

    av_id = test_attendance_victims(
        root_token,
        user_token,
        pm_id,
        city_a,
        users["admin_id"],
        users["user_b_id"],
        victims["victim_id"],
        root_id,
    )
    require_or_exit(av_id, "Cannot proceed without attendance victim")

    ao_id = test_attendance_offenders(
        root_token,
        user_token,
        pm_id,
        city_a,
        users["admin_id"],
        users["user_b_id"],
        offenders["offender_id"],
        victims["victim_id"],
        root_id,
    )
    require_or_exit(ao_id, "Cannot proceed without attendance offender")

    test_attendance_permissions(user_token, pm_id)

    victim_ids = []
    victim_ids.extend(victims.get("victim_ids", []))
    victim_ids.extend(permission_entities.get("victim_ids", []))

    offender_ids = []
    offender_ids.extend(offenders.get("offender_ids", []))
    offender_ids.extend(permission_entities.get("offender_ids", []))

    user_ids = [
        users.get("extra_user_id"),
        users.get("user_b_id"),
        users.get("user_id"),
        users.get("admin_id"),
    ]

    test_cleanup_delete(
        root_token,
        av_id,
        ao_id,
        measures.get("pm_ids", []),
        victim_ids,
        offender_ids,
        city_a,
        city_b,
        user_ids,
    )

    ok = summary()
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
