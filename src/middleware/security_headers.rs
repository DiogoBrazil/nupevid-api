use actix_web::middleware::DefaultHeaders;

/// Security headers applied to every response (registered as the outermost
/// middleware so they also cover 401/403 responses emitted by AuthMiddleware).
/// HSTS is not set here: TLS terminates at the reverse proxy (Traefik), which
/// owns that header.
pub fn security_headers() -> DefaultHeaders {
    DefaultHeaders::new()
        .add(("X-Content-Type-Options", "nosniff"))
        .add(("X-Frame-Options", "DENY"))
        .add(("Cache-Control", "no-store"))
}
