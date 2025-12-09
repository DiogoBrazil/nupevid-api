use actix_web::web;

use crate::controllers::attendance_victims;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/attendance-victims")
            .service(
                web::resource("")
                    .route(web::post().to(attendance_victims::create_attendance_victim))
                    .route(web::get().to(attendance_victims::get_all_attendance_victims)),
            )
            .service(
                web::resource("/{id}")
                    .route(web::get().to(attendance_victims::get_attendance_victim_by_id))
                    .route(web::put().to(attendance_victims::update_attendance_victim_by_id))
                    .route(web::delete().to(attendance_victims::delete_attendance_victim_by_id)),
            ),
    );
}
