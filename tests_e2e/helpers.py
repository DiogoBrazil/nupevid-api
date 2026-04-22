"""Helper functions for E2E API tests."""
import os
import time
from typing import Any, Callable, Optional

import requests


BASE = os.environ.get("NUPEVID_E2E_BASE", "http://localhost:8080/api/v1")
API_KEY = os.environ.get("NUPEVID_E2E_API_KEY", "0eaccfd1fcafe065a4d591458db5d8f1")

# Run ID para evitar conflitos de dados entre execucoes.
RUN_ID = str(time.time()).replace(".", "")[-6:]
UUID_ZERO = "00000000-0000-0000-0000-000000000000"

_stats = {"passed": 0, "failed": 0, "skipped": 0, "errors": [], "skips": []}


def stats():
    return _stats


def run_id():
    return RUN_ID


def headers_api_key(api_key: Optional[str] = None):
    return {"api_key": api_key or API_KEY, "Content-Type": "application/json"}


def headers_auth(token: str, api_key: Optional[str] = None):
    return {
        "api_key": api_key or API_KEY,
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json",
    }


def _record_pass(label):
    _stats["passed"] += 1
    print(f"  ✅ {label}")


def fail(label, details=""):
    _stats["failed"] += 1
    msg = f"❌ {label}"
    if details:
        msg = f"{msg} | {details}"
    _stats["errors"].append(msg)
    print(f"  {msg}")
    return False


def skip(label, details=""):
    _stats["skipped"] += 1
    msg = f"⚠️  SKIP [{label}]"
    if details:
        msg = f"{msg}: {details}"
    _stats["skips"].append(msg)
    print(f"  {msg}")


def pass_check(label):
    _record_pass(label)
    return True


def _body_preview(response):
    if response is None:
        return "no response"
    try:
        body = response.json()
        return body.get("message") or body.get("error") or str(body)[:240]
    except Exception:
        return response.text[:240]


def do(method, path, token=None, json_data=None, params=None, expected=None, label=""):
    """Make a request and optionally assert status code."""
    headers = headers_auth(token) if token else headers_api_key()
    return do_raw(method, path, headers=headers, json_data=json_data, params=params, expected=expected, label=label)


def do_raw(method, path, headers=None, json_data=None, params=None, expected=None, label=""):
    url = f"{BASE}/{path}"
    try:
        response = requests.request(
            method,
            url,
            headers=headers or {},
            json=json_data,
            params=params,
            timeout=15,
        )
    except Exception as exc:
        fail(f"CONN ERROR [{label}]", str(exc))
        return None

    if expected is not None:
        expect_status(response, expected, label)
    return response


def expect_status(response, expected, label):
    if response is None:
        return fail(label, "request returned no response")
    expected_values = expected if isinstance(expected, (tuple, list, set)) else [expected]
    if response.status_code in expected_values:
        _record_pass(f"{label} -> {response.status_code}")
        return True
    return fail(
        label,
        f"got {response.status_code}, expected {list(expected_values)} | {_body_preview(response)}",
    )


def response_json(response, label):
    if response is None:
        fail(label, "request returned no response")
        return None
    try:
        return response.json()
    except Exception as exc:
        fail(label, f"response is not JSON: {exc}")
        return None


def data(response, label="response data"):
    body = response_json(response, label)
    if body is None:
        return None
    if "data" not in body:
        fail(label, f"missing data key in response: {body}")
        return None
    return body["data"]


def require(value, label, details=""):
    if value:
        _record_pass(label)
        return True
    return fail(label, details)


def require_id(value, label):
    return require(bool(value), label, "expected a non-empty id")


def expect_equal(actual, expected, label):
    if actual == expected:
        _record_pass(f"{label} == {expected!r}")
        return True
    return fail(label, f"got {actual!r}, expected {expected!r}")


def expect_not_equal(actual, unexpected, label):
    if actual != unexpected:
        _record_pass(f"{label} != {unexpected!r}")
        return True
    return fail(label, f"unexpected value {unexpected!r}")


def expect_truth(value, label):
    if value:
        _record_pass(label)
        return True
    return fail(label, f"got {value!r}")


def expect_absent(mapping, key, label):
    if isinstance(mapping, dict) and key not in mapping:
        _record_pass(label)
        return True
    return fail(label, f"unexpected key {key!r}")


def expect_field(mapping, field, expected, label=None):
    label = label or f"field {field}"
    if not isinstance(mapping, dict):
        return fail(label, f"expected dict, got {type(mapping).__name__}")
    return expect_equal(mapping.get(field), expected, label)


def expect_policy_contains(user, policy, city_id, label):
    policies = user.get("permission_policies", {}) if isinstance(user, dict) else {}
    cities = policies.get(policy, [])
    if city_id in cities:
        _record_pass(label)
        return True
    return fail(label, f"policy {policy!r} does not contain city {city_id}; got {cities}")


def expect_policy_not_contains(user, policy, city_id, label):
    policies = user.get("permission_policies", {}) if isinstance(user, dict) else {}
    cities = policies.get(policy, [])
    if city_id not in cities:
        _record_pass(label)
        return True
    return fail(label, f"policy {policy!r} still contains city {city_id}")


def _iter_items(collection):
    if isinstance(collection, dict) and "data" in collection:
        collection = collection["data"]
    if collection is None:
        return []
    if isinstance(collection, list):
        return collection
    return []


def find_item(items, predicate: Callable[[dict], bool]):
    for item in _iter_items(items):
        if isinstance(item, dict) and predicate(item):
            return item
    return None


def expect_contains(items, predicate, label, details="item not found"):
    item = find_item(items, predicate)
    if item is not None:
        _record_pass(label)
        return item
    fail(label, details)
    return None


def expect_not_contains(items, predicate, label, details="unexpected item found"):
    item = find_item(items, predicate)
    if item is None:
        _record_pass(label)
        return True
    return fail(label, f"{details}: {item}")


def expect_contains_id(items, item_id, label):
    return expect_contains(items, lambda item: item.get("id") == item_id, label, f"id {item_id} not found")


def expect_not_contains_id(items, item_id, label):
    return expect_not_contains(items, lambda item: item.get("id") == item_id, label, f"id {item_id} found")


def expect_pagination(response, label, min_items=0):
    body = response_json(response, label)
    if body is None:
        return None
    ok = True
    for key in ("data", "page", "page_size", "total_items", "total_pages"):
        ok = expect_truth(key in body, f"{label} has {key}") and ok
    if isinstance(body.get("data"), list) and len(body["data"]) >= min_items:
        _record_pass(f"{label} has at least {min_items} items")
    else:
        ok = fail(label, f"expected at least {min_items} items, got {len(body.get('data', []))}") and ok
    return body if ok else body


def expect_error_message_contains(response, expected_text, label):
    body = response_json(response, label)
    message = ""
    if isinstance(body, dict):
        message = body.get("message") or body.get("error") or ""
    if expected_text in message:
        _record_pass(label)
        return True
    return fail(label, f"message {message!r} does not contain {expected_text!r}")


def login(email, password, auto_session=False, expected=None):
    response = do(
        "POST",
        "auth/login",
        json_data={
            "email": email,
            "password": password,
            "auto_create_session": auto_session,
        },
        expected=expected,
        label=f"login({email})",
    )
    if response and response.status_code == 200:
        payload = data(response, f"login data for {email}") or {}
        return payload.get("token"), payload.get("id"), payload
    if response:
        print(f"    login failed for {email}: {response.status_code} - {_body_preview(response)}")
    return None, None, None


def section(name):
    print(f"\n{'=' * 60}")
    print(f"  {name}")
    print(f"{'=' * 60}")


def summary():
    print(f"\n{'=' * 60}")
    print(
        f"  SUMMARY: {_stats['passed']} passed, {_stats['failed']} failed, {_stats['skipped']} skipped"
    )
    print(f"{'=' * 60}")
    if _stats["errors"]:
        print("\nFailed tests:")
        for error in _stats["errors"]:
            print(f"  {error}")
    if _stats["skips"]:
        print("\nSkipped tests:")
        for item in _stats["skips"]:
            print(f"  {item}")
    return _stats["failed"] == 0


def valid_cpf(seed):
    """Generate a valid masked CPF for deterministic E2E data."""
    digits = [int(ch) for ch in str(seed) if ch.isdigit()]
    if not digits:
        digits = [1, 2, 3, 4, 5, 6]
    while len(digits) < 9:
        digits.extend(digits)
    base_digits = digits[:9]
    if len(set(base_digits)) == 1:
        base_digits[-1] = (base_digits[-1] + 1) % 10

    def check_digit(nums):
        total = sum(num * weight for num, weight in zip(nums, range(len(nums) + 1, 1, -1)))
        digit = 11 - (total % 11)
        return 0 if digit >= 10 else digit

    first = check_digit(base_digits)
    second = check_digit(base_digits + [first])
    nums = base_digits + [first, second]
    return f"{nums[0]}{nums[1]}{nums[2]}.{nums[3]}{nums[4]}{nums[5]}.{nums[6]}{nums[7]}{nums[8]}-{nums[9]}{nums[10]}"


def unique_registration(suffix: int):
    return f"1000{RUN_ID[:4]}{suffix % 10}"
