use crate::controllers::protective_measures;
use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/protective-measures")
            .service(
                web::resource("")
                    .route(web::post().to(protective_measures::create_protective_measure))
                    .route(web::get().to(protective_measures::get_all_protective_measures)),
            )
            .service(
                web::resource("/{id}")
                    .route(web::get().to(protective_measures::get_protective_measure_by_id))
                    .route(web::put().to(protective_measures::update_protective_measure_by_id))
                    .route(web::delete().to(protective_measures::delete_protective_measure_by_id)),
            )
            .service(
                web::resource("/victim/{victim_id}")
                    .route(web::get().to(protective_measures::get_protective_measures_by_victim)),
            ),
    );
}
