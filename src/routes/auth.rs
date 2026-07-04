use crate::controllers::auth;
use crate::middleware::rate_limit::AuthRateLimiterConfig;
use actix_governor::Governor;
use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig, rate_limiter: Option<&AuthRateLimiterConfig>) {
    let scope = web::scope("/auth")
        .service(web::resource("/login").route(web::post().to(auth::login)))
        .service(web::resource("/refresh").route(web::post().to(auth::refresh)))
        .service(web::resource("/logout").route(web::post().to(auth::logout)));

    // Brute-force protection: per-IP rate limit on the auth endpoints.
    // Disabled when the configured limit is 0 (e.g. in tests).
    match rate_limiter {
        Some(config) => cfg.service(scope.wrap(Governor::new(config))),
        None => cfg.service(scope),
    };
}
