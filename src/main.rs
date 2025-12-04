use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use env_logger::{Builder, Env};
use log::info;
use nupevid_api::adapters::{password_hasher::Argon2PasswordHasher, token_generator::JwtTokenGenerator};
use nupevid_api::config::{config_env::Config, database::init_database};
use nupevid_api::middleware::auth::AuthMiddleware;
use nupevid_api::repositories::{
    attendances::PgAttendanceRepository,
    auth::PgAuthRepository,
    cities::PgCityRepository,
    offenders::PgOffenderRepository,
    protective_measures::PgProtectiveMeasureRepository,
    users::PgUserRepository,
    victims::PgVictimRepository,
};
use nupevid_api::routes::config::base_routes::configure_routes;
use nupevid_api::services::{
    attendances::AttendanceService, auth::AuthService, cities::CityService,
    offenders::OffenderService, protective_measures::ProtectiveMeasureService,
    users::UserService, victims::VictimService,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let env = Env::default().filter_or("RUST_LOG", "info,actix_web=info");

    Builder::from_env(env)
        .format_timestamp_millis()
        .format_module_path(true)
        .init();

    dotenv::dotenv().ok();

    let config = Config::from_env();

    let pool = init_database(&config.database_url).await;
    info!("Database connection established");

    // Create adapters
    let password_hasher = Box::new(Argon2PasswordHasher::new());
    let token_generator = Box::new(JwtTokenGenerator::new());
    info!("Adapters created");

    // Create repositories
    let user_repository = web::Data::new(PgUserRepository::new(pool.clone()));
    let auth_repository = web::Data::new(PgAuthRepository::new(pool.clone()));
    let city_repository = web::Data::new(PgCityRepository::new(pool.clone()));
    let victim_repository = web::Data::new(PgVictimRepository::new(pool.clone()));
    let offender_repository = web::Data::new(PgOffenderRepository::new(pool.clone()));
    let protective_measure_repository = web::Data::new(PgProtectiveMeasureRepository::new(pool.clone()));
    let attendance_repository = web::Data::new(PgAttendanceRepository::new(pool.clone()));
    info!("Repositories created");

    // Create services
    let user_service = web::Data::new(UserService::new(
        user_repository.clone(),
        password_hasher.clone(),
    ));
    let auth_service = web::Data::new(AuthService::new(
        auth_repository.clone(),
        web::Data::new(config.clone()),
        password_hasher.clone(),
        token_generator.clone(),
    ));
    let city_service = web::Data::new(CityService::new(city_repository.clone(), user_repository.clone()));
    let victim_service = web::Data::new(VictimService::new(victim_repository.clone(), user_repository.clone()));
    let offender_service = web::Data::new(OffenderService::new(offender_repository.clone(), user_repository.clone()));
    let protective_measure_service = web::Data::new(ProtectiveMeasureService::new(
        protective_measure_repository.clone(),
        victim_repository.clone(),
        user_repository.clone(),
    ));
    let attendance_service = web::Data::new(AttendanceService::new(
        attendance_repository.clone(),
        victim_repository.clone(),
        user_repository.clone(),
    ));
    info!("Services created");

    // Start the server
    let server_addr = config.sercer_addr.clone();
    info!("Server will be started at: http://{}", server_addr);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(AuthMiddleware)
            .app_data(user_repository.clone())
            .app_data(auth_repository.clone())
            .app_data(city_repository.clone())
            .app_data(victim_repository.clone())
            .app_data(offender_repository.clone())
            .app_data(protective_measure_repository.clone())
            .app_data(attendance_repository.clone())
            .app_data(user_service.clone())
            .app_data(auth_service.clone())
            .app_data(city_service.clone())
            .app_data(victim_service.clone())
            .app_data(offender_service.clone())
            .app_data(protective_measure_service.clone())
            .app_data(attendance_service.clone())
            .app_data(web::Data::new(config.clone()))
            .configure(configure_routes)
    })
    .bind(server_addr)?
    .run()
    .await
}
