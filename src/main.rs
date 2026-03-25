use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware::Logger, web};
use env_logger::{Builder, Env};
use log::info;
use nupevid_api::adapters::{
    password_hasher::Argon2PasswordHasher, token_generator::JwtTokenGenerator,
};
use nupevid_api::config::{config_env::Config, database::init_database};
use nupevid_api::core::contracts::adapters::password_hasher::PasswordHasherPort;
use nupevid_api::core::contracts::adapters::token_generator::TokenGeneratorPort;
use nupevid_api::core::contracts::repository::attendance_members::AttendanceMemberRepository;
use nupevid_api::core::contracts::repository::attendance_offenders::{
    AttendanceOffenderReadRepository, AttendanceOffenderWriteRepository,
};
use nupevid_api::core::contracts::repository::attendance_victims::{
    AttendanceVictimReadRepository, AttendanceVictimWriteRepository,
};
use nupevid_api::core::contracts::repository::auth::AuthRepository;
use nupevid_api::core::contracts::repository::cities::CityRepository;
use nupevid_api::core::contracts::repository::extensions::ExtensionRepository;
use nupevid_api::core::contracts::repository::offenders::{
    OffenderReadRepository, OffenderWriteRepository,
};
use nupevid_api::core::contracts::repository::protective_measures::{
    ProtectiveMeasureReadRepository, ProtectiveMeasureWriteRepository,
};
use nupevid_api::core::contracts::repository::users::UserRepository;
use nupevid_api::core::contracts::repository::victims::{
    VictimReadRepository, VictimWriteRepository,
};
use nupevid_api::core::contracts::repository::work_sessions::{
    WorkSessionReadRepository, WorkSessionWriteRepository,
};
use nupevid_api::middleware::auth::AuthMiddleware;
use nupevid_api::repositories::{
    attendance_members::PgAttendanceMemberRepository,
    attendance_offenders::PgAttendanceOffenderRepository,
    attendance_victims::PgAttendanceVictimRepository, auth::PgAuthRepository,
    cities::PgCityRepository, extensions::PgExtensionRepository, offenders::PgOffenderRepository,
    protective_measures::PgProtectiveMeasureRepository, users::PgUserRepository,
    victims::PgVictimRepository, work_sessions::PgWorkSessionRepository,
};
use nupevid_api::routes::config::base_routes::configure_routes;
use nupevid_api::services::{
    attendance_offenders::AttendanceOffenderService,
    attendance_offenders::AttendanceOffenderServiceDeps,
    attendance_victims::AttendanceVictimService, auth::AuthService, cities::CityService,
    extensions::ExtensionService, offenders::OffenderService,
    protective_measures::ProtectiveMeasureService, users::UserService, victims::VictimService,
    work_sessions::WorkSessionService,
};
use std::sync::Arc;

use nupevid_api::utils::seeder::seed_admin_user;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let env = Env::default().filter_or("RUST_LOG", "info,actix_web=info");

    Builder::from_env(env)
        .format_timestamp_millis()
        .format_module_path(true)
        .init();

    dotenv::dotenv().ok();

    let config = Config::from_env().unwrap_or_else(|e| {
        eprintln!("Configuration error: {}", e);
        std::process::exit(1);
    });

    let pool = init_database(&config.database_url, config.db_max_connections).await;
    info!("Database connection established");

    // Create adapters
    let password_hasher: Arc<dyn PasswordHasherPort> = Arc::new(Argon2PasswordHasher::new());
    let token_generator: Arc<dyn TokenGeneratorPort> = Arc::new(JwtTokenGenerator::new());
    info!("Adapters created");

    // Seed admin user
    seed_admin_user(&pool, password_hasher.as_ref()).await;

    // Create repositories
    let user_repository: Arc<dyn UserRepository> = Arc::new(PgUserRepository::new(pool.clone()));
    let auth_repository: Arc<dyn AuthRepository> = Arc::new(PgAuthRepository::new(pool.clone()));
    let city_repository: Arc<dyn CityRepository> = Arc::new(PgCityRepository::new(pool.clone()));
    let victim_repository = Arc::new(PgVictimRepository::new(pool.clone()));
    let victim_read_repository: Arc<dyn VictimReadRepository> = victim_repository.clone();
    let victim_write_repository: Arc<dyn VictimWriteRepository> = victim_repository.clone();
    let offender_repository = Arc::new(PgOffenderRepository::new(pool.clone()));
    let offender_read_repository: Arc<dyn OffenderReadRepository> = offender_repository.clone();
    let offender_write_repository: Arc<dyn OffenderWriteRepository> = offender_repository.clone();
    let protective_measure_repository = Arc::new(PgProtectiveMeasureRepository::new(pool.clone()));
    let protective_measure_read_repository: Arc<dyn ProtectiveMeasureReadRepository> =
        protective_measure_repository.clone();
    let protective_measure_write_repository: Arc<dyn ProtectiveMeasureWriteRepository> =
        protective_measure_repository.clone();
    let extension_repository: Arc<dyn ExtensionRepository> =
        Arc::new(PgExtensionRepository::new(pool.clone()));
    let attendance_victim_repository = Arc::new(PgAttendanceVictimRepository::new(pool.clone()));
    let attendance_victim_read_repository: Arc<dyn AttendanceVictimReadRepository> =
        attendance_victim_repository.clone();
    let attendance_victim_write_repository: Arc<dyn AttendanceVictimWriteRepository> =
        attendance_victim_repository.clone();
    let attendance_offender_repository =
        Arc::new(PgAttendanceOffenderRepository::new(pool.clone()));
    let attendance_offender_read_repository: Arc<dyn AttendanceOffenderReadRepository> =
        attendance_offender_repository.clone();
    let attendance_offender_write_repository: Arc<dyn AttendanceOffenderWriteRepository> =
        attendance_offender_repository.clone();
    let work_session_repository = Arc::new(PgWorkSessionRepository::new(pool.clone()));
    let work_session_read_repository: Arc<dyn WorkSessionReadRepository> =
        work_session_repository.clone();
    let work_session_write_repository: Arc<dyn WorkSessionWriteRepository> =
        work_session_repository.clone();
    let attendance_member_repository: Arc<dyn AttendanceMemberRepository> =
        Arc::new(PgAttendanceMemberRepository::new(pool.clone()));
    info!("Repositories created");

    let config_arc = Arc::new(config.clone());

    // Create services
    let user_service = web::Data::new(UserService::new(
        Arc::clone(&user_repository),
        Arc::clone(&password_hasher),
    ));
    let auth_service = web::Data::new(AuthService::new(
        Arc::clone(&auth_repository),
        Arc::clone(&work_session_read_repository),
        Arc::clone(&work_session_write_repository),
        Arc::clone(&config_arc),
        Arc::clone(&password_hasher),
        Arc::clone(&token_generator),
    ));
    let city_service = web::Data::new(CityService::new(
        Arc::clone(&city_repository),
        Arc::clone(&user_repository),
    ));
    let victim_service = web::Data::new(VictimService::new(
        Arc::clone(&victim_read_repository),
        Arc::clone(&victim_write_repository),
        Arc::clone(&user_repository),
    ));
    let offender_service = web::Data::new(OffenderService::new(
        Arc::clone(&offender_read_repository),
        Arc::clone(&offender_write_repository),
        Arc::clone(&user_repository),
    ));
    let protective_measure_service = web::Data::new(ProtectiveMeasureService::new(
        Arc::clone(&protective_measure_read_repository),
        Arc::clone(&protective_measure_write_repository),
        Arc::clone(&victim_read_repository),
        Arc::clone(&offender_read_repository),
        Arc::clone(&user_repository),
        Arc::clone(&extension_repository),
        Arc::clone(&city_repository),
    ));
    let extension_service = web::Data::new(ExtensionService::new(
        Arc::clone(&extension_repository),
        Arc::clone(&protective_measure_read_repository),
        Arc::clone(&victim_read_repository),
        Arc::clone(&user_repository),
    ));
    let attendance_victim_service = web::Data::new(AttendanceVictimService::new(
        Arc::clone(&attendance_victim_read_repository),
        Arc::clone(&attendance_victim_write_repository),
        Arc::clone(&victim_read_repository),
        Arc::clone(&user_repository),
        Arc::clone(&work_session_read_repository),
        Arc::clone(&attendance_member_repository),
    ));
    let attendance_offender_service = web::Data::new(AttendanceOffenderService::new(
        AttendanceOffenderServiceDeps {
            attendance_offender_read_repository: Arc::clone(&attendance_offender_read_repository),
            attendance_offender_write_repository: Arc::clone(&attendance_offender_write_repository),
            offender_repository: Arc::clone(&offender_read_repository),
            victim_repository: Arc::clone(&victim_read_repository),
            protective_measure_repository: Arc::clone(&protective_measure_read_repository),
            user_repository: Arc::clone(&user_repository),
            work_session_repository: Arc::clone(&work_session_read_repository),
            attendance_member_repository: Arc::clone(&attendance_member_repository),
        },
    ));
    let work_session_service = web::Data::new(WorkSessionService::new(
        Arc::clone(&work_session_read_repository),
        Arc::clone(&work_session_write_repository),
        Arc::clone(&user_repository),
    ));
    info!("Services created");

    // Start the server
    let server_addr = config.server_addr.clone();
    info!("Server will be started at: http://{}", server_addr);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_origin_fn(|origin, _req_head| {
                log::debug!("CORS Origin: {:?}", origin);
                true
            })
            .allow_any_method()
            .allow_any_header()
            .supports_credentials()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(AuthMiddleware)
            .app_data(user_service.clone())
            .app_data(auth_service.clone())
            .app_data(city_service.clone())
            .app_data(victim_service.clone())
            .app_data(offender_service.clone())
            .app_data(protective_measure_service.clone())
            .app_data(extension_service.clone())
            .app_data(attendance_victim_service.clone())
            .app_data(attendance_offender_service.clone())
            .app_data(work_session_service.clone())
            .app_data(web::Data::new(config.clone()))
            .configure(configure_routes)
    })
    .bind(server_addr)?
    .run()
    .await
}
