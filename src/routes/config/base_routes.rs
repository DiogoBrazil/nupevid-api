use actix_web::web;
use crate::routes::{attendances, auth, cities, protective_measures, users, victims};

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .configure(attendances::configure_routes)
            .configure(auth::configure_routes)
            .configure(cities::configure_routes)
            .configure(protective_measures::configure_routes)
            .configure(users::configure_routes)
            .configure(victims::configure_routes)
    );
}
