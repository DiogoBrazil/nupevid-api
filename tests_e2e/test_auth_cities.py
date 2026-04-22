"""Test auth and cities endpoints."""
from helpers import *


def test_auth():
    section("AUTH")

    do_raw(
        "POST",
        "auth/login",
        headers={"Content-Type": "application/json"},
        json_data={"email": "admin@email.com", "password": "admin@123"},
        expected=401,
        label="login without api_key -> 401",
    )

    do_raw(
        "GET",
        "users",
        headers=headers_api_key("wrong-api-key"),
        expected=401,
        label="protected route with wrong api_key -> 401",
    )

    do("GET", "users", expected=401, label="protected route without token -> 401")

    do_raw(
        "GET",
        "users",
        headers={**headers_api_key(), "Authorization": "Bearer invalid-token"},
        expected=401,
        label="protected route with invalid token -> 401",
    )

    do(
        "POST",
        "auth/login",
        json_data={"email": "admin@email.com", "password": "wrong"},
        expected=401,
        label="login wrong password",
    )
    do(
        "POST",
        "auth/login",
        json_data={"email": "nobody@email.com", "password": "abc"},
        expected=401,
        label="login unknown email",
    )
    do(
        "POST",
        "auth/login",
        json_data={"email": "admin@email.com"},
        expected=422,
        label="login missing password -> 422",
    )
    do(
        "POST",
        "auth/login",
        json_data={"email": "admin@email.com", "password": "admin@123", "extra": True},
        expected=422,
        label="login unknown field -> 422",
    )

    response = do(
        "POST",
        "auth/login",
        json_data={
            "email": "admin@email.com",
            "password": "admin@123",
            "auto_create_session": False,
        },
        expected=200,
        label="login ROOT ok",
    )
    payload = data(response, "ROOT login payload") or {}
    token = payload.get("token")
    root_id = payload.get("id")
    require_id(token, "ROOT login returns token")
    require_id(root_id, "ROOT login returns id")
    expect_field(payload, "profile", "ROOT", "ROOT login profile")
    expect_equal(payload.get("work_session"), None, "ROOT login without auto session")

    auto_response = do(
        "POST",
        "auth/login",
        json_data={
            "email": "admin@email.com",
            "password": "admin@123",
            "auto_create_session": True,
        },
        expected=200,
        label="login ROOT auto_create_session -> 200",
    )
    auto_payload = data(auto_response, "ROOT auto login payload") or {}
    expect_truth(auto_payload.get("work_session"), "ROOT auto login returns work_session")
    if auto_payload.get("work_session"):
        expect_equal(
            auto_payload["work_session"].get("is_active"),
            True,
            "ROOT auto login work_session is active",
        )

    return token, root_id


def test_cities(token):
    section("CITIES (ROOT)")

    response = do(
        "POST",
        "cities",
        token,
        json_data={"name": "ARIQUEMES", "state": "RO", "battalion": "7ºBPM"},
        expected=201,
        label="create city A (ARIQUEMES)",
    )
    city_a_data = data(response, "city A create data") or {}
    city_a = city_a_data.get("id")
    require_id(city_a, "city A id exists")
    expect_field(city_a_data, "name", "ARIQUEMES", "city A name")
    expect_field(city_a_data, "state", "RO", "city A state")
    expect_field(city_a_data, "battalion", "7ºBPM", "city A battalion")

    response = do(
        "POST",
        "cities",
        token,
        json_data={"name": "PORTO VELHO", "state": "RO", "battalion": "1ºBPM"},
        expected=201,
        label="create city B (PORTO VELHO)",
    )
    city_b_data = data(response, "city B create data") or {}
    city_b = city_b_data.get("id")
    require_id(city_b, "city B id exists")
    expect_field(city_b_data, "name", "PORTO VELHO", "city B name")

    do(
        "POST",
        "cities",
        token,
        json_data={"name": "ARIQUEMES", "state": "RO", "battalion": "7ºBPM"},
        expected=400,
        label="create duplicate city -> 400",
    )
    do(
        "POST",
        "cities",
        token,
        json_data={"name": "CIDADE FAKE", "state": "RO", "battalion": "1ºBPM"},
        expected=400,
        label="create city invalid name -> 400",
    )
    do(
        "POST",
        "cities",
        token,
        json_data={"name": "VILHENA", "state": "SP", "battalion": "1ºBPM"},
        expected=400,
        label="create city invalid state -> 400",
    )
    do(
        "POST",
        "cities",
        token,
        json_data={"name": "VILHENA", "state": "RO", "battalion": "INVALID"},
        expected=400,
        label="create city invalid battalion -> 400",
    )
    do(
        "POST",
        "cities",
        token,
        json_data={"name": "", "state": "RO", "battalion": "1ºBPM"},
        expected=400,
        label="create city empty name -> 400",
    )
    do(
        "POST",
        "cities",
        token,
        json_data={"name": "VILHENA", "state": "RO", "battalion": "3ºBPM", "extra": True},
        expected=422,
        label="create city unknown field -> 422",
    )

    list_response = do("GET", "cities", token, expected=200, label="list cities")
    list_payload = expect_pagination(list_response, "cities pagination", min_items=2) or {}
    expect_contains_id(list_payload.get("data"), city_a, "list cities contains city A")
    expect_contains_id(list_payload.get("data"), city_b, "list cities contains city B")

    if city_a:
        get_response = do("GET", f"cities/{city_a}", token, expected=200, label="get city A by id")
        get_payload = data(get_response, "city A detail") or {}
        expect_field(get_payload, "id", city_a, "city A detail id")
        expect_field(get_payload, "name", "ARIQUEMES", "city A detail name")

        update_response = do(
            "PUT",
            f"cities/{city_a}",
            token,
            json_data={"name": "ARIQUEMES", "state": "RO", "battalion": "5ºBPM"},
            expected=200,
            label="update city A battalion",
        )
        update_payload = data(update_response, "city A update data") or {}
        expect_field(update_payload, "battalion", "5ºBPM", "city A updated battalion")

        get_response = do("GET", f"cities/{city_a}", token, expected=200, label="get updated city A")
        expect_field(data(get_response, "updated city A data") or {}, "battalion", "5ºBPM", "city A persisted battalion")
        revert_response = do(
            "PUT",
            f"cities/{city_a}",
            token,
            json_data={"name": "ARIQUEMES", "state": "RO", "battalion": "7ºBPM"},
            expected=200,
            label="revert city A battalion",
        )
        expect_field(data(revert_response, "city A revert data") or {}, "battalion", "7ºBPM", "city A reverted battalion")

    do(
        "GET",
        f"cities/{UUID_ZERO}",
        token,
        expected=404,
        label="get city not found -> 404",
    )

    return city_a, city_b


def test_cities_permission(token_admin, token_user, city_a, city_b):
    section("CITIES PERMISSIONS")

    do(
        "POST",
        "cities",
        token_admin,
        json_data={"name": "VILHENA", "state": "RO", "battalion": "3ºBPM"},
        expected=403,
        label="CITY_ADMIN create city -> 403",
    )
    do(
        "PUT",
        f"cities/{city_a}",
        token_admin,
        json_data={"name": "ARIQUEMES", "state": "RO", "battalion": "7ºBPM"},
        expected=403,
        label="CITY_ADMIN update city -> 403",
    )
    do("DELETE", f"cities/{city_a}", token_admin, expected=403, label="CITY_ADMIN delete city -> 403")

    do(
        "POST",
        "cities",
        token_user,
        json_data={"name": "VILHENA", "state": "RO", "battalion": "3ºBPM"},
        expected=403,
        label="CITY_USER create city -> 403",
    )
    do(
        "PUT",
        f"cities/{city_a}",
        token_user,
        json_data={"name": "ARIQUEMES", "state": "RO", "battalion": "7ºBPM"},
        expected=403,
        label="CITY_USER update city -> 403",
    )
    do("DELETE", f"cities/{city_a}", token_user, expected=403, label="CITY_USER delete city -> 403")

    admin_list = do("GET", "cities", token_admin, expected=200, label="CITY_ADMIN list cities")
    admin_payload = data(admin_list, "CITY_ADMIN cities") or []
    expect_contains_id(admin_payload, city_a, "CITY_ADMIN sees own city")
    expect_not_contains_id(admin_payload, city_b, "CITY_ADMIN does not see other city")

    user_list = do("GET", "cities", token_user, expected=200, label="CITY_USER list cities")
    user_payload = data(user_list, "CITY_USER cities") or []
    expect_contains_id(user_payload, city_a, "CITY_USER sees own city")
    expect_not_contains_id(user_payload, city_b, "CITY_USER does not see other city")
