use crate::controllers::users;
use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .service(
                web::resource("")
                    .route(web::post().to(users::create_user))
                    .route(web::get().to(users::get_all_users)),
            )
            .service(
                web::resource("/{id}")
                    .route(web::put().to(users::update_user_by_id))
                    .route(web::get().to(users::get_user_by_id))
                    .route(web::delete().to(users::delete_user_by_id)),
            )
            .service(
                web::resource("/{id}/password")
                    .route(web::patch().to(users::update_user_password_by_id)),
            )
            .service(
                web::resource("/{id}/password/reset")
                    .route(web::post().to(users::reset_user_password_by_id)),
            )
            .service(
                web::resource("/{id}/policies/{policy}/cities")
                    .route(web::post().to(users::append_user_policy_cities))
                    .route(web::delete().to(users::remove_user_policy_cities)),
            ),
    );
}
