use actix_web::{http::StatusCode, test};
use sqlx::PgPool;

use crate::common::{db_fixtures, test_helpers};

#[sqlx::test]
async fn root_can_get_machine_information(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let root_claims = test_helpers::build_root_claims();
    let root_token = test_helpers::generate_jwt(&root_claims, &config.jwt_secret);
    let req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/machine-information"),
        &config,
        &root_token,
    )
    .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Security headers devem estar presentes em todas as respostas
    let headers = resp.headers();
    assert_eq!(
        headers.get("X-Content-Type-Options").unwrap(),
        "nosniff",
        "missing X-Content-Type-Options header"
    );
    assert_eq!(
        headers.get("X-Frame-Options").unwrap(),
        "DENY",
        "missing X-Frame-Options header"
    );
    assert_eq!(
        headers.get("Cache-Control").unwrap(),
        "no-store",
        "missing Cache-Control header"
    );
}

#[sqlx::test]
async fn non_root_cannot_get_machine_information(pool: PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let city_id = db_fixtures::insert_city(&pool, "PORTO VELHO").await;

    // CITY_ADMIN -> FORBIDDEN
    let admin_claims = test_helpers::seed_city_admin_claims(&pool, city_id).await;
    let admin_token = test_helpers::generate_jwt(&admin_claims, &config.jwt_secret);
    let admin_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/machine-information"),
        &config,
        &admin_token,
    )
    .to_request();
    let admin_resp = test::call_service(&app, admin_req).await;
    assert_eq!(admin_resp.status(), StatusCode::FORBIDDEN);

    // CITY_USER -> FORBIDDEN
    let user_claims = test_helpers::seed_city_user_claims(&pool, city_id).await;
    let user_token = test_helpers::generate_jwt(&user_claims, &config.jwt_secret);
    let user_req = test_helpers::with_auth_headers(
        test::TestRequest::get().uri("/api/v1/machine-information"),
        &config,
        &user_token,
    )
    .to_request();
    let user_resp = test::call_service(&app, user_req).await;
    assert_eq!(user_resp.status(), StatusCode::FORBIDDEN);
}
