"""Test victims and offenders CRUD."""
from helpers import *


def _cpf(slot):
    return valid_cpf(f"{RUN_ID}{slot}")


def _victim_payload(name, city_id=None, cpf=None, addresses=None):
    payload = {
        "full_name": name,
        "uses_alcohol": False,
        "uses_drugs": False,
    }
    if cpf is not None:
        payload["cpf"] = cpf
    if city_id is not None:
        payload["city_id"] = city_id
    if addresses is not None:
        payload["addresses"] = addresses
    return payload


def _full_victim_payload(name, cpf, city_id):
    return {
        "full_name": name,
        "cpf": cpf,
        "birth_date": "1990-05-15",
        "city_id": city_id,
        "phones": [{"phone": "(69) 99999-8888", "phone_type": "Mobile"}],
        "addresses": [
            {
                "street": "Rua das Flores",
                "number": "123",
                "district": "Centro",
                "city_id": city_id,
                "zip_code": "76801-000",
                "address_type": "Residential",
            }
        ],
        "education_level": "High School",
        "occupation": "Professora",
        "has_children": True,
        "children_count": 2,
        "is_pregnant": False,
        "uses_alcohol": False,
        "uses_drugs": False,
    }


def _offender_payload(name, city_id=None, cpf=None, addresses=None):
    payload = {
        "full_name": name,
        "imprisoned": False,
        "uses_alcohol": False,
        "uses_drugs": False,
        "education_level": "High School",
    }
    if cpf is not None:
        payload["cpf"] = cpf
    if city_id is not None:
        payload["city_id"] = city_id
    if addresses is not None:
        payload["addresses"] = addresses
    return payload


def _full_offender_payload(name, cpf, city_id):
    return {
        "full_name": name,
        "cpf": cpf,
        "birth_date": "1980-05-20",
        "city_id": city_id,
        "imprisoned": False,
        "occupation": "Mecanico",
        "uses_alcohol": True,
        "uses_drugs": False,
        "education_level": "High School",
        "phones": [{"phone": "(69) 99999-7777", "phone_type": "Mobile"}],
        "addresses": [
            {
                "street": "Rua A",
                "number": "10",
                "district": "Centro",
                "city_id": city_id,
                "zip_code": "76801-000",
                "address_type": "Residential",
            }
        ],
    }


def _residential_address(city_id):
    return {
        "street": "Rua Residencial",
        "number": "100",
        "district": "Centro",
        "city_id": city_id,
        "zip_code": "76801-000",
        "address_type": "Residential",
    }


def test_victims_crud(root_token, city_a, city_b):
    section("VICTIMS CRUD (ROOT)")
    ctx = {"victim_ids": []}
    victim_cpf = _cpf(11)
    victim_b_cpf = _cpf(12)

    response = do(
        "POST",
        "victims",
        root_token,
        json_data=_full_victim_payload(f"Ana Paula dos Santos {RUN_ID}", victim_cpf, city_a),
        expected=201,
        label="create victim",
    )
    victim = data(response, "victim create data") or {}
    victim_id = victim.get("id")
    ctx["victim_id"] = victim_id
    ctx["victim_ids"].append(victim_id)
    orig_addr_id = victim.get("addresses", [{}])[0].get("id") if victim.get("addresses") else None
    orig_phone_id = victim.get("phones", [{}])[0].get("id") if victim.get("phones") else None
    require_id(victim_id, "victim id exists")
    expect_field(victim, "cpf", victim_cpf, "victim cpf")
    expect_field(victim, "city_id", city_a, "victim city_id")
    expect_equal(len(victim.get("phones", [])), 1, "victim initial phone count")
    expect_equal(len(victim.get("addresses", [])), 1, "victim initial address count")

    do("POST", "victims", root_token, json_data=_victim_payload("Dup CPF", city_a, victim_cpf), expected=409, label="create victim duplicate CPF -> 409")
    do("POST", "victims", root_token, json_data=_victim_payload("CPF Invalido", city_a, "000.000.000-00"), expected=400, label="create victim invalid CPF -> 400")
    do("POST", "victims", root_token, json_data=_victim_payload("CPF Sem Mascara", city_a, victim_cpf.replace(".", "").replace("-", "")), expected=400, label="create victim CPF without mask -> 400")
    do("POST", "victims", root_token, json_data={"uses_alcohol": False, "uses_drugs": False}, expected=422, label="create victim missing name -> 422")
    do("POST", "victims", root_token, json_data=_victim_payload("   ", city_a), expected=400, label="create victim blank name -> 400")
    do("POST", "victims", root_token, json_data=_victim_payload("No City"), expected=400, label="create victim without city or address -> 400")
    do("POST", "victims", root_token, json_data={**_victim_payload("Unknown Field", city_a), "extra": True}, expected=422, label="create victim unknown field -> 422")

    derived_response = do(
        "POST",
        "victims",
        root_token,
        json_data=_victim_payload(f"Vitima Cidade Derivada {RUN_ID}", cpf=_cpf(13), addresses=[_residential_address(city_a)]),
        expected=201,
        label="create victim derives city from residential address",
    )
    derived = data(derived_response, "derived victim data") or {}
    ctx["victim_derived_id"] = derived.get("id")
    ctx["victim_ids"].append(ctx["victim_derived_id"])
    expect_field(derived, "city_id", city_a, "derived victim city_id")

    city_b_response = do(
        "POST",
        "victims",
        root_token,
        json_data=_victim_payload(f"Vitima Porto Velho {RUN_ID}", city_b, victim_b_cpf),
        expected=201,
        label="create victim city B",
    )
    victim_b = data(city_b_response, "victim city B data") or {}
    ctx["victim_b_id"] = victim_b.get("id")
    ctx["victim_ids"].append(ctx["victim_b_id"])
    expect_field(victim_b, "city_id", city_b, "victim city B city_id")

    list_response = do("GET", "victims", root_token, expected=200, label="list victims")
    list_payload = expect_pagination(list_response, "victims pagination", min_items=3) or {}
    expect_contains_id(list_payload.get("data"), victim_id, "list victims contains main victim")

    search_name = do("GET", "victims/search", root_token, params={"name": "Ana Paula"}, expected=200, label="search victims by name")
    expect_contains_id(data(search_name, "victim search by name data"), victim_id, "search victims by name finds main victim")
    search_cpf = do("GET", "victims/search", root_token, params={"cpf": victim_cpf}, expected=200, label="search victims by cpf")
    expect_contains_id(data(search_cpf, "victim search by cpf data"), victim_id, "search victims by cpf finds main victim")
    do("GET", "victims/search", root_token, expected=400, label="search victims without criteria -> 400")
    do("GET", "victims/search", root_token, params={"name": "Ana", "cpf": victim_cpf}, expected=400, label="search victims with two criteria -> 400")

    if victim_id:
        detail = do("GET", f"victims/{victim_id}", root_token, expected=200, label="get victim by id")
        expect_field(data(detail, "victim detail data") or {}, "id", victim_id, "victim detail id")

        phone_response = do(
            "POST",
            f"victims/{victim_id}/phones",
            root_token,
            json_data={"phone": "(69) 98888-7777", "phone_type": "Residential"},
            expected=201,
            label="add phone to victim",
        )
        phone = data(phone_response, "victim phone create data") or {}
        new_phone_id = phone.get("id")
        require_id(new_phone_id, "new victim phone id exists")
        expect_field(phone, "phone_type", "Residential", "new victim phone type")

        if new_phone_id:
            update_phone = do(
                "PUT",
                f"victims/{victim_id}/phones/{new_phone_id}",
                root_token,
                json_data={"phone": "(69) 97777-6666", "phone_type": "Work"},
                expected=200,
                label="update victim phone",
            )
            expect_field(data(update_phone, "victim phone update data") or {}, "phone_type", "Work", "victim phone type updated")
            do("DELETE", f"victims/{victim_id}/phones/{new_phone_id}", root_token, expected=200, label="delete victim phone")
            detail = do("GET", f"victims/{victim_id}", root_token, expected=200, label="get victim after phone delete")
            expect_not_contains_id((data(detail, "victim after phone delete data") or {}).get("phones"), new_phone_id, "deleted victim phone absent")

        if orig_addr_id:
            update_addr = do(
                "PUT",
                f"victims/{victim_id}/addresses/{orig_addr_id}",
                root_token,
                json_data={
                    "street": "Rua Atualizada",
                    "number": "456",
                    "district": "Centro",
                    "city_id": city_a,
                    "zip_code": "76801-111",
                    "address_type": "Residential",
                },
                expected=200,
                label="update victim address",
            )
            expect_field(data(update_addr, "victim address update data") or {}, "street", "Rua Atualizada", "victim address street updated")
            do("DELETE", f"victims/{victim_id}/addresses/{orig_addr_id}", root_token, expected=200, label="delete victim address")

        addr_response = do(
            "POST",
            f"victims/{victim_id}/addresses",
            root_token,
            json_data={
                "street": "Av. Brasil",
                "number": "500",
                "district": "Centro",
                "city_id": city_a,
                "zip_code": "76820-000",
                "address_type": "Work",
            },
            expected=201,
            label="add address to victim",
        )
        new_addr = data(addr_response, "victim address create data") or {}
        expect_field(new_addr, "address_type", "Work", "new victim address type")

        updated_victim_name = f"Ana Paula Atualizada {RUN_ID}".upper()
        update_response = do(
            "PUT",
            f"victims/{victim_id}",
            root_token,
            json_data=_victim_payload(f"Ana Paula Atualizada {RUN_ID}", city_a, victim_cpf),
            expected=200,
            label="update victim",
        )
        expect_field(data(update_response, "victim update data") or {}, "full_name", updated_victim_name, "victim name updated")
        detail = do("GET", f"victims/{victim_id}", root_token, expected=200, label="get updated victim")
        expect_field(data(detail, "updated victim detail") or {}, "full_name", updated_victim_name, "victim name persisted")

    if orig_phone_id:
        do("PUT", f"victims/{victim_id}/phones/{UUID_ZERO}", root_token, json_data={"phone": "(69) 90000-0000", "phone_type": "Mobile"}, expected=404, label="update missing victim phone -> 404")
    do("GET", f"victims/{UUID_ZERO}", root_token, expected=404, label="get victim not found -> 404")

    return ctx


def test_offenders_crud(root_token, city_a, city_b):
    section("OFFENDERS CRUD (ROOT)")
    ctx = {"offender_ids": []}
    offender_cpf = _cpf(21)
    offender_b_cpf = _cpf(22)

    response = do(
        "POST",
        "offenders",
        root_token,
        json_data=_full_offender_payload(f"Joao Agressor {RUN_ID}", offender_cpf, city_a),
        expected=201,
        label="create offender",
    )
    offender = data(response, "offender create data") or {}
    offender_id = offender.get("id")
    ctx["offender_id"] = offender_id
    ctx["offender_ids"].append(offender_id)
    off_phone_id = offender.get("phones", [{}])[0].get("id") if offender.get("phones") else None
    off_addr_id = offender.get("addresses", [{}])[0].get("id") if offender.get("addresses") else None
    require_id(offender_id, "offender id exists")
    expect_field(offender, "cpf", offender_cpf, "offender cpf")
    expect_field(offender, "city_id", city_a, "offender city_id")

    do(
        "POST",
        "offenders",
        root_token,
        json_data=_offender_payload("Dup CPF", city_a, offender_cpf),
        expected=409,
        label="create offender duplicate CPF -> 409",
    )
    do("POST", "offenders", root_token, json_data=_offender_payload("CPF Bad", city_a, "000.000.000-00"), expected=400, label="create offender invalid CPF -> 400")
    do("POST", "offenders", root_token, json_data=_offender_payload("CPF Sem Mascara", city_a, offender_cpf.replace(".", "").replace("-", "")), expected=400, label="create offender CPF without mask -> 400")
    do("POST", "offenders", root_token, json_data={"uses_alcohol": False}, expected=422, label="create offender missing fields -> 422")
    do("POST", "offenders", root_token, json_data=_offender_payload("   ", city_a), expected=400, label="create offender blank name -> 400")
    do("POST", "offenders", root_token, json_data=_offender_payload("No City"), expected=400, label="create offender without city or address -> 400")
    do("POST", "offenders", root_token, json_data={**_offender_payload("Bad Enum", city_a), "education_level": "Invalid"}, expected=422, label="create offender invalid enum -> 422")

    derived_response = do(
        "POST",
        "offenders",
        root_token,
        json_data=_offender_payload(f"Agressor Cidade Derivada {RUN_ID}", cpf=_cpf(23), addresses=[_residential_address(city_a)]),
        expected=201,
        label="create offender derives city from residential address",
    )
    derived = data(derived_response, "derived offender data") or {}
    ctx["offender_derived_id"] = derived.get("id")
    ctx["offender_ids"].append(ctx["offender_derived_id"])
    expect_field(derived, "city_id", city_a, "derived offender city_id")

    city_b_response = do(
        "POST",
        "offenders",
        root_token,
        json_data=_offender_payload(f"Agressor Porto Velho {RUN_ID}", city_b, offender_b_cpf),
        expected=201,
        label="create offender city B",
    )
    offender_b = data(city_b_response, "offender city B data") or {}
    ctx["offender_b_id"] = offender_b.get("id")
    ctx["offender_ids"].append(ctx["offender_b_id"])
    expect_field(offender_b, "city_id", city_b, "offender city B city_id")

    list_response = do("GET", "offenders", root_token, expected=200, label="list offenders")
    list_payload = expect_pagination(list_response, "offenders pagination", min_items=3) or {}
    expect_contains_id(list_payload.get("data"), offender_id, "list offenders contains main offender")

    search_name = do("GET", "offenders/search", root_token, params={"name": "Joao Agressor"}, expected=200, label="search offenders by name")
    expect_contains_id(data(search_name, "offender search by name data"), offender_id, "search offenders by name finds main offender")
    search_cpf = do("GET", "offenders/search", root_token, params={"cpf": offender_cpf}, expected=200, label="search offenders by cpf")
    expect_contains_id(data(search_cpf, "offender search by cpf data"), offender_id, "search offenders by cpf finds main offender")
    do("GET", "offenders/search", root_token, expected=400, label="search offenders without criteria -> 400")

    if offender_id:
        detail = do("GET", f"offenders/{offender_id}", root_token, expected=200, label="get offender by id")
        expect_field(data(detail, "offender detail data") or {}, "id", offender_id, "offender detail id")

        phone_response = do(
            "POST",
            f"offenders/{offender_id}/phones",
            root_token,
            json_data={"phone": "(69) 98888-5555", "phone_type": "Work"},
            expected=201,
            label="add phone to offender",
        )
        new_phone = data(phone_response, "offender phone create data") or {}
        expect_field(new_phone, "phone_type", "Work", "new offender phone type")

        if off_phone_id:
            update_phone = do(
                "PUT",
                f"offenders/{offender_id}/phones/{off_phone_id}",
                root_token,
                json_data={"phone": "(69) 97777-4444", "phone_type": "Residential"},
                expected=200,
                label="update offender phone",
            )
            expect_field(data(update_phone, "offender phone update data") or {}, "phone_type", "Residential", "offender phone type updated")
            do("DELETE", f"offenders/{offender_id}/phones/{off_phone_id}", root_token, expected=200, label="delete offender phone")
            detail = do("GET", f"offenders/{offender_id}", root_token, expected=200, label="get offender after phone delete")
            expect_not_contains_id((data(detail, "offender after phone delete data") or {}).get("phones"), off_phone_id, "deleted offender phone absent")

        if off_addr_id:
            update_addr = do(
                "PUT",
                f"offenders/{offender_id}/addresses/{off_addr_id}",
                root_token,
                json_data={
                    "street": "Rua Atualizada",
                    "number": "789",
                    "district": "Centro",
                    "city_id": city_a,
                    "zip_code": "76801-222",
                    "address_type": "Residential",
                },
                expected=200,
                label="update offender address",
            )
            expect_field(data(update_addr, "offender address update data") or {}, "street", "Rua Atualizada", "offender address street updated")
            do("DELETE", f"offenders/{offender_id}/addresses/{off_addr_id}", root_token, expected=200, label="delete offender address")

        addr_response = do(
            "POST",
            f"offenders/{offender_id}/addresses",
            root_token,
            json_data={
                "street": "Av. Mamore",
                "number": "300",
                "district": "Embratel",
                "city_id": city_a,
                "zip_code": "76820-000",
                "address_type": "Work",
            },
            expected=201,
            label="add address to offender",
        )
        expect_field(data(addr_response, "offender address create data") or {}, "address_type", "Work", "new offender address type")

        updated_offender_name = f"Joao Agressor Updated {RUN_ID}".upper()
        update_response = do(
            "PUT",
            f"offenders/{offender_id}",
            root_token,
            json_data=_offender_payload(f"Joao Agressor Updated {RUN_ID}", city_a, offender_cpf),
            expected=200,
            label="update offender",
        )
        expect_field(data(update_response, "offender update data") or {}, "full_name", updated_offender_name, "offender name updated")
        detail = do("GET", f"offenders/{offender_id}", root_token, expected=200, label="get updated offender")
        expect_field(data(detail, "updated offender detail") or {}, "full_name", updated_offender_name, "offender name persisted")

    do("GET", f"offenders/{UUID_ZERO}", root_token, expected=404, label="get offender not found -> 404")

    return ctx


def test_offenders_by_victim(root_token, victim_id, offender_id):
    section("OFFENDERS BY VICTIM")

    response = do("GET", f"offenders/victim/{victim_id}", root_token, expected=200, label="get offenders by victim")
    expect_contains_id(data(response, "offenders by victim data"), offender_id, "offenders by victim contains linked offender")


def test_victims_offenders_permissions(admin_token, user_token, victim_id, offender_id, city_a, city_b):
    section("VICTIMS/OFFENDERS PERMISSIONS")
    created = {"victim_ids": [], "offender_ids": []}

    admin_victims = do("GET", "victims", admin_token, expected=200, label="CITY_ADMIN list victims -> 200")
    admin_victim_data = data(admin_victims, "CITY_ADMIN victims") or []
    expect_contains_id(admin_victim_data, victim_id, "CITY_ADMIN sees own city victim")

    admin_offenders = do("GET", "offenders", admin_token, expected=200, label="CITY_ADMIN list offenders -> 200")
    admin_offender_data = data(admin_offenders, "CITY_ADMIN offenders") or []
    expect_contains_id(admin_offender_data, offender_id, "CITY_ADMIN sees own city offender")

    admin_victim = do(
        "POST",
        "victims",
        admin_token,
        json_data=_victim_payload(f"Vitima by Admin {RUN_ID}", city_a),
        expected=201,
        label="CITY_ADMIN create victim own city -> 201",
    )
    admin_victim_id = (data(admin_victim, "admin-created victim data") or {}).get("id")
    created["victim_ids"].append(admin_victim_id)

    admin_offender = do(
        "POST",
        "offenders",
        admin_token,
        json_data=_offender_payload(f"Agressor by Admin {RUN_ID}", city_a),
        expected=201,
        label="CITY_ADMIN create offender own city -> 201",
    )
    admin_offender_id = (data(admin_offender, "admin-created offender data") or {}).get("id")
    created["offender_ids"].append(admin_offender_id)

    do(
        "POST",
        "victims",
        admin_token,
        json_data=_victim_payload(f"Vitima Admin Cidade B {RUN_ID}", city_b),
        expected=403,
        label="CITY_ADMIN create victim other city -> 403",
    )
    do(
        "POST",
        "offenders",
        admin_token,
        json_data=_offender_payload(f"Agressor Admin Cidade B {RUN_ID}", city_b),
        expected=403,
        label="CITY_ADMIN create offender other city -> 403",
    )

    user_victims = do("GET", "victims", user_token, expected=200, label="CITY_USER list victims -> 200")
    expect_contains_id(data(user_victims, "CITY_USER victims") or [], victim_id, "CITY_USER sees own city victim")

    user_victim = do(
        "POST",
        "victims",
        user_token,
        json_data=_victim_payload(f"Vitima by User {RUN_ID}", city_a),
        expected=201,
        label="CITY_USER with create_victims own city -> 201",
    )
    user_victim_id = (data(user_victim, "user-created victim data") or {}).get("id")
    created["victim_ids"].append(user_victim_id)

    do(
        "POST",
        "victims",
        user_token,
        json_data=_victim_payload(f"Vitima User Cidade B {RUN_ID}", city_b),
        expected=403,
        label="CITY_USER create_victims other city -> 403",
    )
    do(
        "POST",
        "offenders",
        user_token,
        json_data=_offender_payload(f"Agressor by User {RUN_ID}", city_a),
        expected=403,
        label="CITY_USER create offender without policy -> 403",
    )

    if victim_id:
        do(
            "PUT",
            f"victims/{victim_id}",
            user_token,
            json_data=_victim_payload("Blocked", city_a),
            expected=403,
            label="CITY_USER update victim no policy -> 403",
        )
        do("DELETE", f"victims/{victim_id}", user_token, expected=403, label="CITY_USER delete victim no policy -> 403")

    if offender_id:
        do(
            "PUT",
            f"offenders/{offender_id}",
            user_token,
            json_data=_offender_payload("Blocked", city_a),
            expected=403,
            label="CITY_USER update offender no policy -> 403",
        )
        do("DELETE", f"offenders/{offender_id}", user_token, expected=403, label="CITY_USER delete offender no policy -> 403")

    return created
