use actix_web::web;

use crate::controllers::work_sessions;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/work-sessions")
            .service(
                web::resource("")
                    .route(web::post().to(work_sessions::create_work_session))
                    .route(web::get().to(work_sessions::list_sessions))
            )
            .route("/active", web::get().to(work_sessions::get_active_session))
            .route("/end", web::post().to(work_sessions::end_session))
            .route("/{id}", web::get().to(work_sessions::get_session_by_id))
            .route("/{id}/members", web::post().to(work_sessions::add_member))
            .route("/{id}/members/{member_id}", web::delete().to(work_sessions::remove_member))
            .route("/{id}/members", web::put().to(work_sessions::update_members))
            .route("/{id}/members/{user_id}/function", web::put().to(work_sessions::update_member_function)),
    );
}
