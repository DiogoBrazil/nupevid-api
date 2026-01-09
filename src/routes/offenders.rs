use actix_web::web;

use crate::controllers::offenders;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/offenders")
            .service(
                web::resource("")
                    .route(web::post().to(offenders::create_offender))
                    .route(web::get().to(offenders::get_all_offenders)),
            )
            .service(web::resource("/search").route(web::get().to(offenders::search_offenders)))
            .service(
                web::resource("/victim/{victim_id}")
                    .route(web::get().to(offenders::get_offenders_by_victim_id)),
            )
            .service(
                web::resource("/{id}")
                    .route(web::get().to(offenders::get_offender_by_id))
                    .route(web::put().to(offenders::update_offender_by_id))
                    .route(web::delete().to(offenders::delete_offender_by_id)),
            )
            .service(
                web::resource("/{id}/phones")
                    .route(web::post().to(offenders::add_phone_to_offender)),
            )
            .service(
                web::resource("/{id}/phones/{phone_id}")
                    .route(web::put().to(offenders::update_offender_phone))
                    .route(web::delete().to(offenders::delete_offender_phone)),
            )
            .service(
                web::resource("/{id}/addresses")
                    .route(web::post().to(offenders::add_address_to_offender)),
            )
            .service(
                web::resource("/{id}/addresses/{address_id}")
                    .route(web::put().to(offenders::update_offender_address))
                    .route(web::delete().to(offenders::delete_offender_address)),
            ),
    );
}
