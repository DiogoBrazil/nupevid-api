use crate::controllers::cities;
use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/cities")
            .service(
                web::resource("")
                    .route(web::post().to(cities::create_city))
                    .route(web::get().to(cities::get_all_cities)),
            )
            .service(
                web::resource("/{id}")
                    .route(web::get().to(cities::get_city_by_id))
                    .route(web::put().to(cities::update_city_by_id))
                    .route(web::delete().to(cities::delete_city_by_id)),
            ),
    );
}
