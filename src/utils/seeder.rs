use crate::core::contracts::adapters::password_hasher::PasswordHasherPort;
use crate::core::contracts::repository::users::UserRepository;
use crate::core::entities::users::CreateUser;
use crate::repositories::users::PgUserRepository;
use log::{error, info};
use sqlx::PgPool;

pub async fn seed_admin_user(pool: &PgPool, hasher: &dyn PasswordHasherPort) {
    let repo = PgUserRepository::new(pool.clone());
    let email = "admin@email.com";

    match repo.check_user_exists_by_email(email).await {
        Ok(exists) => {
            if !exists {
                info!("Admin user not found. Creating default admin user...");
                let password_hash = match hasher.hash_password("admin@123") {
                    Ok(h) => h,
                    Err(e) => {
                        error!("Failed to hash password for admin user: {}", e);
                        return;
                    }
                };

                let admin = CreateUser {
                    rank: "ST PM".to_string(),
                    registration: "1000012345".to_string(),
                    full_name: "admin".to_string(),
                    profile: "ROOT".to_string(),
                    email: email.to_string(),
                    password: password_hash,
                    city_id: None,
                    permission_policies: None,
                };

                match repo.create_user(admin).await {
                    Ok(user) => info!("Admin user created successfully with ID: {}", user.id),
                    Err(e) => error!("Failed to create admin user: {}", e),
                }
            } else {
                info!("Admin user already exists.");
            }
        }
        Err(e) => error!("Failed to check if admin user exists: {}", e),
    }
}
