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
            )
            .service(
                web::resource("/{id}/members")
                    .route(web::get().to(attendance_victims::get_attendance_members))
                    .route(web::post().to(attendance_victims::add_attendance_member)),
            )
            .service(
                web::resource("/{id}/members/{user_id}")
                    .route(web::delete().to(attendance_victims::remove_attendance_member)),
            )
            .service(
                web::resource("/by-measure/{protective_measure_id}")
                    .route(web::get().to(attendance_victims::get_attendance_victims_by_measure)),
            )
            .service(
                web::resource("/by-victim/{victim_id}")
                    .route(web::get().to(attendance_victims::get_attendance_victims_by_victim)),
            ),
    );
}
