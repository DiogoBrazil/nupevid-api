use actix_web::web;
use crate::routes::{attendances, auth, cities, offenders, protective_measures, swagger, users, victims};

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .configure(swagger::configure_routes)
            .service(
                web::scope("/v1")
                    .configure(attendances::configure_routes)
                    .configure(auth::configure_routes)
                    .configure(cities::configure_routes)
                    .configure(offenders::configure_routes)
                    .configure(protective_measures::configure_routes)
                    .configure(users::configure_routes)
                    .configure(victims::configure_routes)
            )
    );
}
