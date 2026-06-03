use crate::controllers::auth;
use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .service(web::resource("/login").route(web::post().to(auth::login)))
            .service(web::resource("/refresh").route(web::post().to(auth::refresh)))
            .service(web::resource("/logout").route(web::post().to(auth::logout))),
    );
}
