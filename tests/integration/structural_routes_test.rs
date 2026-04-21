use actix_web::{http::StatusCode, test as aw_test};
use sqlx::PgPool;

use crate::common::test_helpers;

use super::route_policy_matrix::ROUTE_POLICY_MATRIX;

fn method(value: &str) -> actix_web::http::Method {
    value.parse().expect("route matrix method must be valid")
}

#[sqlx::test]
async fn all_registered_routes_require_authentication_except_explicit_public_list(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool, config.clone()).await;

    for case in ROUTE_POLICY_MATRIX.iter().filter(|case| !case.is_public) {
        let req = aw_test::TestRequest::default()
            .method(method(case.method))
            .uri(case.path)
            .insert_header(("api_key", config.api_key.clone()))
            .to_request();

        let err = aw_test::try_call_service(&app, req)
            .await
            .expect_err("protected route should reject missing Authorization");

        assert_eq!(
            err.as_response_error().status_code(),
            StatusCode::UNAUTHORIZED,
            "{} {} should reject missing Authorization",
            case.method,
            case.path
        );
    }
}

#[test]
fn all_registered_routes_have_expected_policy_mapping() {
    for case in ROUTE_POLICY_MATRIX.iter().filter(|case| !case.is_public) {
        assert!(
            case.expected_policy.is_some() || case.expected_profile.is_some(),
            "{} {} must document either a policy or an explicit profile/auth requirement",
            case.method,
            case.path
        );
    }
}
