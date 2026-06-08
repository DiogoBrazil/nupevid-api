"""Test users endpoints and permission management."""
from helpers import *


def _email(prefix):
    return f"{prefix}_{RUN_ID}@email.com"


def _user_payload(rank, registration, full_name, profile, email, city_id=None, password="senha@123", policies=None):
    payload = {
        "rank": rank,
        "registration": registration,
        "full_name": full_name,
        "profile": profile,
        "email": email,
        "password": password,
    }
    if city_id is not None:
        payload["city_id"] = city_id
    if policies is not None:
        payload["permission_policies"] = policies
    return payload


def _update_payload(rank, registration, full_name, profile, email, city_id=None, policies=None):
    payload = {
        "rank": rank,
        "registration": registration,
        "full_name": full_name,
        "profile": profile,
        "email": email,
    }
    if city_id is not None:
        payload["city_id"] = city_id
    if policies is not None:
        payload["permission_policies"] = policies
    return payload


def test_users_crud(root_token, city_a, city_b):
    section("USERS CRUD (ROOT)")

    users = {
        "admin_email": _email("admin"),
        "user_email": _email("user"),
        "user_b_email": _email("userpv"),
        "admin_reg": unique_registration(1),
        "user_reg": unique_registration(2),
        "user_b_reg": unique_registration(3),
    }

    admin_response = do(
        "POST",
        "users",
        root_token,
        json_data=_user_payload(
            "CAP PM",
            users["admin_reg"],
            f"Admin Ariquemes {RUN_ID}",
            "CITY_ADMIN",
            users["admin_email"],
            city_a,
        ),
        expected=201,
        label="create CITY_ADMIN for city A",
    )
    admin = data(admin_response, "CITY_ADMIN create data") or {}
    users["admin_id"] = admin.get("id")
    require_id(users["admin_id"], "CITY_ADMIN id exists")
    expect_field(admin, "profile", "CITY_ADMIN", "CITY_ADMIN profile")
    expect_field(admin, "city_id", city_a, "CITY_ADMIN city")
    expect_absent(admin, "password", "CITY_ADMIN response does not expose password")
    expect_policy_contains(admin, "create_users", city_a, "CITY_ADMIN default create_users policy")
    expect_policy_contains(admin, "delete_protective_measures", city_a, "CITY_ADMIN default delete PM policy")

    do(
        "POST",
        "users",
        root_token,
        json_data=_user_payload(
            "CAP PM",
            unique_registration(4),
            f"Admin2 Ariquemes {RUN_ID}",
            "CITY_ADMIN",
            _email("admin2"),
            city_a,
        ),
        expected=400,
        label="create 2nd CITY_ADMIN same city -> 400",
    )

    user_response = do(
        "POST",
        "users",
        root_token,
        json_data=_user_payload(
            "SD PM",
            users["user_reg"],
            f"User Ariquemes {RUN_ID}",
            "CITY_USER",
            users["user_email"],
            city_a,
        ),
        expected=201,
        label="create CITY_USER for city A",
    )
    user = data(user_response, "CITY_USER create data") or {}
    users["user_id"] = user.get("id")
    require_id(users["user_id"], "CITY_USER id exists")
    expect_field(user, "profile", "CITY_USER", "CITY_USER profile")
    expect_policy_contains(user, "read_victims", city_a, "CITY_USER default read_victims policy")
    expect_policy_contains(user, "read_attendances", city_a, "CITY_USER default read_attendances policy")
    expect_policy_not_contains(user, "create_victims", city_a, "CITY_USER does not have create_victims by default")
    expect_absent(user, "password", "CITY_USER response does not expose password")

    user_b_response = do(
        "POST",
        "users",
        root_token,
        json_data=_user_payload(
            "SD PM",
            users["user_b_reg"],
            f"User Porto Velho {RUN_ID}",
            "CITY_USER",
            users["user_b_email"],
            city_b,
        ),
        expected=201,
        label="create CITY_USER for city B",
    )
    user_b = data(user_b_response, "CITY_USER city B create data") or {}
    users["user_b_id"] = user_b.get("id")
    require_id(users["user_b_id"], "CITY_USER city B id exists")
    expect_field(user_b, "city_id", city_b, "CITY_USER city B city_id")

    do(
        "POST",
        "users",
        root_token,
        json_data=_user_payload(
            "SD PM",
            unique_registration(5),
            "Dup Email",
            "CITY_USER",
            users["user_email"],
            city_a,
        ),
        expected=400,
        label="create user duplicate email -> 400",
    )
    do(
        "POST",
        "users",
        root_token,
        json_data=_user_payload(
            "SD PM",
            users["user_reg"],
            "Dup Reg",
            "CITY_USER",
            _email("dupreg"),
            city_a,
        ),
        expected=400,
        label="create user duplicate registration -> 400",
    )
    do(
        "POST",
        "users",
        root_token,
        json_data=_user_payload("SD PM", "999999999", "Bad Reg", "CITY_USER", _email("badreg"), city_a),
        expected=400,
        label="create user invalid registration -> 400",
    )
    do(
        "POST",
        "users",
        root_token,
        json_data=_user_payload("SD PM", unique_registration(6), "Bad Email", "CITY_USER", "invalid-email", city_a),
        expected=400,
        label="create user invalid email -> 400",
    )
    do(
        "POST",
        "users",
        root_token,
        json_data=_user_payload("GENERAL", unique_registration(7), "Bad Rank", "CITY_USER", _email("badrank"), city_a),
        expected=422,
        label="create user invalid rank -> 422",
    )
    do(
        "POST",
        "users",
        root_token,
        json_data=_user_payload("SD PM", unique_registration(8), "No City", "CITY_USER", _email("nocity")),
        expected=400,
        label="create CITY_USER without city -> 400",
    )
    do(
        "POST",
        "users",
        root_token,
        json_data={**_user_payload("SD PM", unique_registration(9), "Unknown Field", "CITY_USER", _email("unknown"), city_a), "extra": True},
        expected=422,
        label="create user unknown field -> 422",
    )

    list_response = do("GET", "users", root_token, expected=200, label="list users (ROOT)")
    list_payload = expect_pagination(list_response, "users pagination", min_items=3) or {}
    expect_contains_id(list_payload.get("data"), users["admin_id"], "ROOT list contains admin")
    expect_contains_id(list_payload.get("data"), users["user_b_id"], "ROOT list contains city B user")

    search_name = do("GET", "users/search", root_token, params={"name": f"Admin Ariquemes {RUN_ID}"}, expected=200, label="search users by name")
    expect_contains_id(data(search_name, "search users by name data"), users["admin_id"], "search by name finds created admin")
    search_reg = do("GET", "users/search", root_token, params={"registration": users["user_reg"]}, expected=200, label="search users by registration")
    expect_contains_id(data(search_reg, "search users by registration data"), users["user_id"], "search by registration finds user")
    do("GET", "users/search", root_token, expected=400, label="search users without criteria -> 400")
    do("GET", "users/search", root_token, params={"name": "x", "registration": users["user_reg"]}, expected=400, label="search users with two criteria -> 400")

    if users["admin_id"]:
        get_response = do("GET", f"users/{users['admin_id']}", root_token, expected=200, label="get user by id")
        get_user = data(get_response, "user detail data") or {}
        expect_field(get_user, "id", users["admin_id"], "get user detail id")
        expect_absent(get_user, "password", "get user detail does not expose password")

    if users["user_id"]:
        user_policies = user.get("permission_policies") or {}
        update_response = do(
            "PUT",
            f"users/{users['user_id']}",
            root_token,
            json_data=_update_payload(
                "CB PM",
                users["user_reg"],
                f"User Ariquemes Updated {RUN_ID}",
                "CITY_USER",
                users["user_email"],
                city_a,
                policies=user_policies,
            ),
            expected=200,
            label="update user",
        )
        updated = data(update_response, "updated user data") or {}
        expect_field(updated, "rank", "CB PM", "updated user rank")
        expect_field(updated, "full_name", f"User Ariquemes Updated {RUN_ID}", "updated user name")
        expect_policy_contains(updated, "read_cities", city_a, "updated user preserved read_cities policy")
        expect_policy_contains(updated, "read_victims", city_a, "updated user preserved read_victims policy")
        get_updated = do("GET", f"users/{users['user_id']}", root_token, expected=200, label="get updated user")
        persisted_user = data(get_updated, "get updated user data") or {}
        expect_field(persisted_user, "rank", "CB PM", "updated user rank persisted")
        expect_policy_contains(persisted_user, "read_cities", city_a, "updated user persisted read_cities policy")
        expect_policy_contains(persisted_user, "read_victims", city_a, "updated user persisted read_victims policy")

    return users


def test_users_permissions(root_token, admin_token, user_token, users, city_a, city_b):
    section("USERS PERMISSIONS")

    admin_id = users["admin_id"]
    user_id = users["user_id"]
    user_b_id = users["user_b_id"]

    admin_list = do("GET", "users", admin_token, expected=200, label="CITY_ADMIN list users -> 200")
    admin_payload = data(admin_list, "CITY_ADMIN users") or []
    expect_contains_id(admin_payload, admin_id, "CITY_ADMIN list includes own city admin")
    expect_contains_id(admin_payload, user_id, "CITY_ADMIN list includes own city user")
    expect_not_contains_id(admin_payload, user_b_id, "CITY_ADMIN list excludes city B user")

    do("GET", "users", user_token, expected=200, label="CITY_USER list users -> 200")

    do(
        "POST",
        "users",
        user_token,
        json_data=_user_payload("SD PM", unique_registration(4), "Denied", "CITY_USER", _email("denied"), city_a),
        expected=403,
        label="CITY_USER create user -> 403",
    )
    do(
        "POST",
        "users",
        admin_token,
        json_data=_user_payload("SD PM", unique_registration(5), "Admin Creates Root", "ROOT", _email("root_by_admin")),
        expected=403,
        label="CITY_ADMIN create ROOT -> 403",
    )
    do(
        "POST",
        "users",
        admin_token,
        json_data=_user_payload("CAP PM", unique_registration(6), "Admin Creates Admin", "CITY_ADMIN", _email("admin_by_admin"), city_a),
        expected=403,
        label="CITY_ADMIN create CITY_ADMIN -> 403",
    )

    extra_response = do(
        "POST",
        "users",
        admin_token,
        json_data=_user_payload(
            "SD PM",
            unique_registration(7),
            f"Created by Admin {RUN_ID}",
            "CITY_USER",
            _email("extra"),
            city_b,
        ),
        expected=201,
        label="CITY_ADMIN create CITY_USER forces own city -> 201",
    )
    extra_user = data(extra_response, "extra user data") or {}
    extra_user_id = extra_user.get("id")
    require_id(extra_user_id, "extra user id exists")
    expect_field(extra_user, "city_id", city_a, "CITY_ADMIN-created user forced to admin city")

    do(
        "PUT",
        f"users/{user_b_id}",
        admin_token,
        json_data=_update_payload(
            "SD PM",
            users["user_b_reg"],
            f"Blocked City B Update {RUN_ID}",
            "CITY_USER",
            users["user_b_email"],
            city_b,
        ),
        expected=403,
        label="CITY_ADMIN update user from other city -> 403",
    )
    do("GET", f"users/{user_b_id}", admin_token, expected=403, label="CITY_ADMIN get user from other city -> 403")

    append_create = do(
        "POST",
        f"users/{user_id}/policies/create_victims/cities",
        root_token,
        json_data={"city_ids": [city_a]},
        expected=200,
        label="ROOT append create_victims policy to user",
    )
    expect_policy_contains(data(append_create, "append create_victims data") or {}, "create_victims", city_a, "create_victims added")

    append_read_b = do(
        "POST",
        f"users/{user_id}/policies/read_victims/cities",
        root_token,
        json_data={"city_ids": [city_b]},
        expected=200,
        label="ROOT append read_victims city_b to user",
    )
    expect_policy_contains(data(append_read_b, "append read_victims data") or {}, "read_victims", city_b, "read_victims city B added")

    admin_append = do(
        "POST",
        f"users/{user_id}/policies/read_offenders/cities",
        admin_token,
        json_data={"city_ids": [city_a]},
        expected=200,
        label="CITY_ADMIN append read_offenders to CITY_USER -> 200",
    )
    expect_policy_contains(data(admin_append, "admin append policy data") or {}, "read_offenders", city_a, "read_offenders city A added")

    do(
        "POST",
        f"users/{user_id}/policies/read_offenders/cities",
        admin_token,
        json_data={"city_ids": [city_b]},
        expected=403,
        label="CITY_ADMIN append policy for city not possessed -> 403",
    )
    do(
        "POST",
        f"users/{user_id}/policies/not_a_policy/cities",
        root_token,
        json_data={"city_ids": [city_a]},
        expected=400,
        label="ROOT append invalid policy -> 400",
    )
    do(
        "POST",
        f"users/{user_id}/policies/create_cities/cities",
        root_token,
        json_data={"city_ids": [city_a]},
        expected=400,
        label="ROOT append non-assignable policy -> 400",
    )

    if extra_user_id:
        do(
            "POST",
            f"users/{extra_user_id}/policies/read_victims/cities",
            user_token,
            json_data={"city_ids": [city_a]},
            expected=403,
            label="CITY_USER append policy -> 403",
        )

    remove_read_b = do(
        "DELETE",
        f"users/{user_id}/policies/read_victims/cities",
        root_token,
        json_data={"city_ids": [city_b]},
        expected=200,
        label="ROOT remove read_victims city_b from user",
    )
    expect_policy_not_contains(data(remove_read_b, "remove read_victims data") or {}, "read_victims", city_b, "read_victims city B removed")

    return extra_user_id


def test_password(admin_token, user_token, users, current_pw="senha@123"):
    section("PASSWORD")

    user_id = users["user_id"]
    user_b_id = users["user_b_id"]
    user_email = users["user_email"]

    update_response = do(
        "PATCH",
        "users/password",
        user_token,
        json_data={"current_password": current_pw, "new_password": "novaSenha@123"},
        expected=200,
        label="update own password -> 200",
    )
    expect_absent(data(update_response, "password update response") or {}, "password", "password update response does not expose password")

    login(user_email, current_pw, expected=401)
    new_token, _, _ = login(user_email, "novaSenha@123", expected=200)
    require_id(new_token, "login with changed password returns token")

    do(
        "PATCH",
        "users/password",
        new_token or user_token,
        json_data={"current_password": "wrong", "new_password": "novaSenha@456"},
        expected=400,
        label="update password wrong current -> 400",
    )
    do(
        "PATCH",
        "users/password",
        new_token or user_token,
        json_data={"new_password": "novaSenha@456"},
        expected=400,
        label="update password missing current -> 400",
    )

    do(
        "POST",
        f"users/{user_b_id}/password/reset",
        admin_token,
        expected=403,
        label="CITY_ADMIN reset password cross-city -> 403",
    )
    do(
        "POST",
        f"users/{user_id}/password/reset",
        new_token or user_token,
        expected=403,
        label="CITY_USER reset password -> 403",
    )

    reset_response = do(
        "POST",
        f"users/{user_id}/password/reset",
        admin_token,
        expected=200,
        label="CITY_ADMIN reset user password -> 200",
    )
    reset_payload = data(reset_response, "password reset data") or {}
    temp_pw = reset_payload.get("temporary_password")
    require(temp_pw, "reset returns temporary_password", "temporary_password missing")

    if temp_pw:
        temp_token, _, _ = login(user_email, temp_pw, expected=200)
        require_id(temp_token, "login with temporary password returns token")
        if temp_token:
            do(
                "PATCH",
                "users/password",
                temp_token,
                json_data={"current_password": temp_pw, "new_password": "senha@123"},
                expected=200,
                label="change temporary password to known password -> 200",
            )

    return "senha@123"
