use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware::Logger, web};
use env_logger::{Builder, Env};
use log::info;
use nupevid_api::app_factory::AppDependencies;
use nupevid_api::config::{config_env::Config, database::init_database};
use nupevid_api::middleware::auth::AuthMiddleware;
use nupevid_api::utils::seeder::seed_admin_user;
use std::io::Write;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let env = Env::default().filter_or("RUST_LOG", "info,actix_web=info");

    Builder::from_env(env)
        .format(|buf, record| {
            let line = serde_json::json!({
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "level": record.level().to_string().to_lowercase(),
                "target": record.target(),
                "module": record.module_path().unwrap_or(""),
                "message": record.args().to_string(),
            });
            writeln!(buf, "{}", line)
        })
        .init();

    dotenv::dotenv().ok();

    let config = Config::from_env().unwrap_or_else(|e| {
        eprintln!("Configuration error: {}", e);
        std::process::exit(1);
    });

    let pool = init_database(&config.database_url, config.db_max_connections).await;
    info!("Database connection established");

    if config.run_migrations_on_startup {
        info!("Running database migrations...");
        if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
            eprintln!("Migration error: {}", e);
            std::process::exit(1);
        }
        info!("Database migrations applied");
    }

    let deps = Arc::new(AppDependencies::new(pool.clone(), config.clone()));
    info!("Dependencies created");

    if config.enable_bootstrap_root {
        seed_admin_user(&pool, deps.password_hasher.as_ref()).await;
    }

    let server_addr = config.server_addr.clone();
    info!("Server will be started at: http://{}", server_addr);

    HttpServer::new(move || {
        let deps = Arc::clone(&deps);
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
            .wrap(AuthMiddleware)
            .wrap(Logger::default())
            .configure(|cfg: &mut web::ServiceConfig| deps.configure(cfg))
    })
    .bind(server_addr)?
    .run()
    .await
}
