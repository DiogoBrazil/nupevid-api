use actix_web::web;
use crate::controllers::attendances;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/attendances")
            .service(web::resource("")
                .route(web::post().to(attendances::create_attendance))
                .route(web::get().to(attendances::get_all_attendances)))
            .service(web::resource("/{id}")
                .route(web::get().to(attendances::get_attendance_by_id))
                .route(web::put().to(attendances::update_attendance_by_id))
                .route(web::delete().to(attendances::delete_attendance_by_id)))
            .service(web::resource("/{id}/address")
                .route(web::get().to(attendances::get_attendance_address)))
    );
    cfg.service(
        web::scope("/attendance-addresses")
            .service(web::resource("")
                .route(web::post().to(attendances::create_attendance_address)))
            .service(web::resource("/{id}")
                .route(web::put().to(attendances::update_attendance_address)))
    );
}
