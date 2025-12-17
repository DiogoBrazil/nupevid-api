use actix_web::web;

use crate::controllers::attendance_offenders;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/attendance-offenders")
            .service(
                web::resource("")
                    .route(web::post().to(attendance_offenders::create_attendance_offender))
                    .route(web::get().to(attendance_offenders::get_all_attendance_offenders)),
            )
            .service(
                web::resource("/{id}")
                    .route(web::get().to(attendance_offenders::get_attendance_offender_by_id))
                    .route(web::put().to(attendance_offenders::update_attendance_offender_by_id))
                    .route(web::delete().to(attendance_offenders::delete_attendance_offender_by_id)),
            )
            .service(
                web::resource("/{id}/members")
                    .route(web::get().to(attendance_offenders::get_attendance_members))
                    .route(web::post().to(attendance_offenders::add_attendance_member)),
            )
            .service(
                web::resource("/{id}/members/{user_id}")
                    .route(web::delete().to(attendance_offenders::remove_attendance_member)),
            )
            .service(
                web::resource("/by-offender/{offender_id}")
                    .route(web::get().to(attendance_offenders::get_attendance_offenders_by_offender)),
            )
            .service(
                web::resource("/by-victim/{victim_id}")
                    .route(web::get().to(attendance_offenders::get_attendance_offenders_by_victim)),
            ),
    );
}
