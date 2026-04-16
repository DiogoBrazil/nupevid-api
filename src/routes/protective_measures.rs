use crate::controllers::{extensions, protective_measures};
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
            )
            .service(
                web::resource("/{id}/extensions")
                    .route(web::post().to(extensions::create_extension))
                    .route(web::get().to(extensions::get_extensions_by_measure)),
            )
            .service(
                web::resource("/{id}/extensions/{ext_id}")
                    .route(web::get().to(extensions::get_extension_by_id))
                    .route(web::put().to(extensions::update_extension_by_id))
                    .route(web::delete().to(extensions::delete_extension_by_id)),
            ),
    );
}
