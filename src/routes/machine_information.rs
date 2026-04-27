use crate::controllers::machine_information;
use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/machine-information").service(
            web::resource("")
                .route(web::get().to(machine_information::get_machine_information)),
        ),
    );
}
