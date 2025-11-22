use actix_web::web;

use crate::controllers::victims;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/victims")
            .service(
                web::resource("")
                    .route(web::post().to(victims::create_victim))
                    .route(web::get().to(victims::get_all_victims)),
            )
            .service(
                web::resource("/{id}")
                    .route(web::get().to(victims::get_victim_by_id))
                    .route(web::put().to(victims::update_victim_by_id))
                    .route(web::delete().to(victims::delete_victim_by_id)),
            ),
    );
}
