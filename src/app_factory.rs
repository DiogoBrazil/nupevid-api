use actix_web::web;
use sqlx::PgPool;
use std::sync::Arc;

use crate::adapters::password_hasher::Argon2PasswordHasher;
use crate::adapters::token_generator::JwtTokenGenerator;
use crate::config::config_env::Config;
use crate::core::contracts::adapters::password_hasher::PasswordHasherPort;
use crate::core::contracts::adapters::token_generator::TokenGeneratorPort;
use crate::core::contracts::repository::attendance_members::AttendanceMemberRepository;
use crate::core::contracts::repository::attendance_offenders::{
    AttendanceOffenderReadRepository, AttendanceOffenderWriteRepository,
};
use crate::core::contracts::repository::attendance_victims::{
    AttendanceVictimReadRepository, AttendanceVictimWriteRepository,
};
use crate::core::contracts::repository::auth::AuthRepository;
use crate::core::contracts::repository::cities::CityRepository;
use crate::core::contracts::repository::extensions::ExtensionRepository;
use crate::core::contracts::repository::offenders::{
    OffenderReadRepository, OffenderWriteRepository,
};
use crate::core::contracts::repository::protective_measures::{
    ProtectiveMeasureReadRepository, ProtectiveMeasureWriteRepository,
};
use crate::core::contracts::repository::users::UserRepository;
use crate::core::contracts::repository::victims::{VictimReadRepository, VictimWriteRepository};
use crate::core::contracts::repository::work_sessions::{
    WorkSessionReadRepository, WorkSessionWriteRepository,
};
use crate::presenters::protective_measures::ProtectiveMeasurePresenter;
use crate::presenters::work_sessions::WorkSessionPresenter;
use crate::repositories::{
    attendance_members::PgAttendanceMemberRepository,
    attendance_offenders::PgAttendanceOffenderRepository,
    attendance_victims::PgAttendanceVictimRepository, auth::PgAuthRepository,
    cities::PgCityRepository, extensions::PgExtensionRepository, offenders::PgOffenderRepository,
    protective_measures::PgProtectiveMeasureRepository, users::PgUserRepository,
    victims::PgVictimRepository, work_sessions::PgWorkSessionRepository,
};
use crate::routes::api_router::configure_routes;
use crate::usecases::attendance_offenders::{
    AddAttendanceOffenderMemberUseCase, AttendanceOffenderUseCaseDependencies,
    CreateAttendanceOffenderUseCase, DeleteAttendanceOffenderUseCase,
    GetAllAttendanceOffendersUseCase, GetAttendanceOffenderByIdUseCase,
    GetAttendanceOffenderMembersUseCase, GetAttendanceOffendersByOffenderUseCase,
    GetAttendanceOffendersByVictimUseCase, RemoveAttendanceOffenderMemberUseCase,
    UpdateAttendanceOffenderUseCase,
};
use crate::usecases::attendance_victims::{
    AddAttendanceMemberUseCase, AttendanceVictimUseCaseDependencies, CreateAttendanceVictimUseCase,
    DeleteAttendanceVictimUseCase, GetAllAttendanceVictimsUseCase, GetAttendanceMembersUseCase,
    GetAttendanceVictimByIdUseCase, GetAttendanceVictimsByVictimUseCase,
    RemoveAttendanceMemberUseCase, UpdateAttendanceVictimUseCase,
};
use crate::usecases::auth::{AuthUseCaseDependencies, LoginUseCase};
use crate::usecases::cities::{
    CityUseCaseDependencies, CreateCityUseCase, DeleteCityByIdUseCase, GetAllCitiesUseCase,
    GetCityByIdUseCase, UpdateCityByIdUseCase,
};
use crate::usecases::extensions::{
    CreateExtensionUseCase, DeleteExtensionByIdUseCase, ExtensionUseCaseDependencies,
    GetExtensionByIdUseCase, GetExtensionsByMeasureUseCase, UpdateExtensionByIdUseCase,
};
use crate::usecases::offenders::{
    CreateOffenderAddressUseCase, CreateOffenderPhoneUseCase, CreateOffenderUseCase,
    DeleteOffenderAddressUseCase, DeleteOffenderPhoneUseCase, DeleteOffenderUseCase,
    GetAllOffendersUseCase, GetOffenderByIdUseCase, GetOffendersByVictimUseCase,
    OffenderUseCaseDependencies, SearchOffendersUseCase, UpdateOffenderAddressUseCase,
    UpdateOffenderPhoneUseCase, UpdateOffenderUseCase,
};
use crate::usecases::protective_measures::{
    CreateProtectiveMeasureUseCase, DeleteProtectiveMeasureUseCase,
    GetAllProtectiveMeasuresUseCase, GetMeasuresByVictimUseCase, GetProtectiveMeasureByIdUseCase,
    ProtectiveMeasureUseCaseDependencies, UpdateProtectiveMeasureUseCase,
};
use crate::usecases::users::{
    AppendUserPolicyCitiesUseCase, CreateUserUseCase, DeleteUserByIdUseCase, GetAllUsersUseCase,
    GetUserByIdUseCase, RemoveUserPolicyCitiesUseCase, ResetUserPasswordByIdUseCase,
    SearchUsersUseCase, UpdateUserPasswordUseCase, UpdateUserUseCase, UserUseCaseDependencies,
};
use crate::usecases::victims::{
    CreateVictimAddressUseCase, CreateVictimPhoneUseCase, CreateVictimUseCase,
    DeleteVictimAddressUseCase, DeleteVictimPhoneUseCase, DeleteVictimUseCase,
    GetAllVictimsUseCase, GetVictimByIdUseCase, SearchVictimsUseCase, UpdateVictimAddressUseCase,
    UpdateVictimPhoneUseCase, UpdateVictimUseCase, VictimUseCaseDependencies,
};
use crate::usecases::work_sessions::{
    AddMemberToSessionUseCase, CreateWorkSessionUseCase, EndSessionUseCase,
    GetActiveSessionUseCase, GetSessionByIdUseCase, ListSessionsUseCase,
    RemoveMemberFromSessionUseCase, UpdateMemberFunctionUseCase, UpdateMembersUseCase,
    UpdateWorkSessionUseCase, WorkSessionUseCaseDependencies,
};

pub struct AppDependencies {
    pub password_hasher: Arc<dyn PasswordHasherPort>,
    config: Config,
    // Use cases stored as web::Data for efficient cloning into App
    usecases: Vec<Box<dyn AppDataRegistrar>>,
}

trait AppDataRegistrar: Send + Sync {
    fn register(&self, cfg: &mut web::ServiceConfig);
}

struct TypedAppData<T: Send + Sync + 'static>(web::Data<T>);

impl<T: Send + Sync + 'static> AppDataRegistrar for TypedAppData<T> {
    fn register(&self, cfg: &mut web::ServiceConfig) {
        cfg.app_data(self.0.clone());
    }
}

impl AppDependencies {
    pub fn new(pool: PgPool, config: Config) -> Self {
        let password_hasher: Arc<dyn PasswordHasherPort> = Arc::new(Argon2PasswordHasher::new());
        let token_generator: Arc<dyn TokenGeneratorPort> = Arc::new(JwtTokenGenerator::new());

        // Repositories
        let user_repository: Arc<dyn UserRepository> =
            Arc::new(PgUserRepository::new(pool.clone()));
        let auth_repository: Arc<dyn AuthRepository> =
            Arc::new(PgAuthRepository::new(pool.clone()));
        let city_repository: Arc<dyn CityRepository> =
            Arc::new(PgCityRepository::new(pool.clone()));
        let victim_repository = Arc::new(PgVictimRepository::new(pool.clone()));
        let victim_read_repository: Arc<dyn VictimReadRepository> = victim_repository.clone();
        let victim_write_repository: Arc<dyn VictimWriteRepository> = victim_repository.clone();
        let offender_repository = Arc::new(PgOffenderRepository::new(pool.clone()));
        let offender_read_repository: Arc<dyn OffenderReadRepository> = offender_repository.clone();
        let offender_write_repository: Arc<dyn OffenderWriteRepository> =
            offender_repository.clone();
        let protective_measure_repository =
            Arc::new(PgProtectiveMeasureRepository::new(pool.clone()));
        let protective_measure_read_repository: Arc<dyn ProtectiveMeasureReadRepository> =
            protective_measure_repository.clone();
        let protective_measure_write_repository: Arc<dyn ProtectiveMeasureWriteRepository> =
            protective_measure_repository.clone();
        let extension_repository: Arc<dyn ExtensionRepository> =
            Arc::new(PgExtensionRepository::new(pool.clone()));
        let attendance_victim_repository =
            Arc::new(PgAttendanceVictimRepository::new(pool.clone()));
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

        let config_arc = Arc::new(config.clone());

        // Use case dependencies
        let user_usecase_deps = UserUseCaseDependencies::new(
            Arc::clone(&user_repository),
            Arc::clone(&password_hasher),
        );
        let city_usecase_deps = CityUseCaseDependencies::new(
            Arc::clone(&city_repository),
            Arc::clone(&user_repository),
        );
        let auth_usecase_deps = AuthUseCaseDependencies::new(
            Arc::clone(&auth_repository),
            Arc::clone(&work_session_read_repository),
            Arc::clone(&work_session_write_repository),
            Arc::clone(&config_arc),
            Arc::clone(&password_hasher),
            Arc::clone(&token_generator),
        );
        let work_session_usecase_deps = WorkSessionUseCaseDependencies::new(
            Arc::clone(&work_session_read_repository),
            Arc::clone(&work_session_write_repository),
            Arc::clone(&user_repository),
        );
        let victim_usecase_deps = VictimUseCaseDependencies::new(
            Arc::clone(&victim_read_repository),
            Arc::clone(&victim_write_repository),
            Arc::clone(&user_repository),
        );
        let offender_usecase_deps = OffenderUseCaseDependencies::new(
            Arc::clone(&offender_read_repository),
            Arc::clone(&offender_write_repository),
            Arc::clone(&user_repository),
        );
        let pm_usecase_deps = ProtectiveMeasureUseCaseDependencies::new(
            Arc::clone(&protective_measure_read_repository),
            Arc::clone(&protective_measure_write_repository),
            Arc::clone(&victim_read_repository),
            Arc::clone(&offender_read_repository),
            Arc::clone(&user_repository),
            Arc::clone(&extension_repository),
            Arc::clone(&city_repository),
        );
        let extension_usecase_deps = ExtensionUseCaseDependencies::new(
            Arc::clone(&extension_repository),
            Arc::clone(&protective_measure_read_repository),
            Arc::clone(&victim_read_repository),
            Arc::clone(&user_repository),
        );
        let attendance_victim_usecase_deps = AttendanceVictimUseCaseDependencies::new(
            Arc::clone(&attendance_victim_read_repository),
            Arc::clone(&attendance_victim_write_repository),
            Arc::clone(&victim_read_repository),
            Arc::clone(&protective_measure_read_repository),
            Arc::clone(&user_repository),
            Arc::clone(&work_session_read_repository),
            Arc::clone(&attendance_member_repository),
        );
        let attendance_offender_usecase_deps = AttendanceOffenderUseCaseDependencies::new(
            Arc::clone(&attendance_offender_read_repository),
            Arc::clone(&attendance_offender_write_repository),
            Arc::clone(&offender_read_repository),
            Arc::clone(&victim_read_repository),
            Arc::clone(&protective_measure_read_repository),
            Arc::clone(&user_repository),
            Arc::clone(&work_session_read_repository),
            Arc::clone(&attendance_member_repository),
        );

        // Build use cases
        let mut usecases: Vec<Box<dyn AppDataRegistrar>> = Vec::new();

        macro_rules! register {
            ($uc:expr) => {
                usecases.push(Box::new(TypedAppData(web::Data::new($uc))));
            };
        }

        // Users
        register!(CreateUserUseCase::new(user_usecase_deps.clone()));
        register!(UpdateUserUseCase::new(user_usecase_deps.clone()));
        register!(GetUserByIdUseCase::new(user_usecase_deps.clone()));
        register!(GetAllUsersUseCase::new(user_usecase_deps.clone()));
        register!(SearchUsersUseCase::new(user_usecase_deps.clone()));
        register!(DeleteUserByIdUseCase::new(user_usecase_deps.clone()));
        register!(UpdateUserPasswordUseCase::new(user_usecase_deps.clone()));
        register!(ResetUserPasswordByIdUseCase::new(user_usecase_deps.clone()));
        register!(AppendUserPolicyCitiesUseCase::new(
            user_usecase_deps.clone()
        ));
        register!(RemoveUserPolicyCitiesUseCase::new(
            user_usecase_deps.clone()
        ));

        // Cities
        register!(CreateCityUseCase::new(city_usecase_deps.clone()));
        register!(GetCityByIdUseCase::new(city_usecase_deps.clone()));
        register!(GetAllCitiesUseCase::new(city_usecase_deps.clone()));
        register!(UpdateCityByIdUseCase::new(city_usecase_deps.clone()));
        register!(DeleteCityByIdUseCase::new(city_usecase_deps.clone()));

        // Auth
        register!(LoginUseCase::new(auth_usecase_deps));

        // Work sessions
        register!(CreateWorkSessionUseCase::new(
            work_session_usecase_deps.clone()
        ));
        register!(GetActiveSessionUseCase::new(
            work_session_usecase_deps.clone()
        ));
        register!(GetSessionByIdUseCase::new(
            work_session_usecase_deps.clone()
        ));
        register!(ListSessionsUseCase::new(work_session_usecase_deps.clone()));
        register!(EndSessionUseCase::new(work_session_usecase_deps.clone()));
        register!(AddMemberToSessionUseCase::new(
            work_session_usecase_deps.clone()
        ));
        register!(RemoveMemberFromSessionUseCase::new(
            work_session_usecase_deps.clone()
        ));
        register!(UpdateMembersUseCase::new(work_session_usecase_deps.clone()));
        register!(UpdateMemberFunctionUseCase::new(
            work_session_usecase_deps.clone()
        ));
        register!(UpdateWorkSessionUseCase::new(
            work_session_usecase_deps.clone()
        ));

        // Victims
        register!(CreateVictimUseCase::new(victim_usecase_deps.clone()));
        register!(GetVictimByIdUseCase::new(victim_usecase_deps.clone()));
        register!(GetAllVictimsUseCase::new(victim_usecase_deps.clone()));
        register!(SearchVictimsUseCase::new(victim_usecase_deps.clone()));
        register!(UpdateVictimUseCase::new(victim_usecase_deps.clone()));
        register!(DeleteVictimUseCase::new(victim_usecase_deps.clone()));
        register!(CreateVictimPhoneUseCase::new(victim_usecase_deps.clone()));
        register!(UpdateVictimPhoneUseCase::new(victim_usecase_deps.clone()));
        register!(DeleteVictimPhoneUseCase::new(victim_usecase_deps.clone()));
        register!(CreateVictimAddressUseCase::new(victim_usecase_deps.clone()));
        register!(UpdateVictimAddressUseCase::new(victim_usecase_deps.clone()));
        register!(DeleteVictimAddressUseCase::new(victim_usecase_deps.clone()));

        // Offenders
        register!(CreateOffenderUseCase::new(offender_usecase_deps.clone()));
        register!(GetOffenderByIdUseCase::new(offender_usecase_deps.clone()));
        register!(GetAllOffendersUseCase::new(offender_usecase_deps.clone()));
        register!(SearchOffendersUseCase::new(offender_usecase_deps.clone()));
        register!(GetOffendersByVictimUseCase::new(
            offender_usecase_deps.clone()
        ));
        register!(UpdateOffenderUseCase::new(offender_usecase_deps.clone()));
        register!(DeleteOffenderUseCase::new(offender_usecase_deps.clone()));
        register!(CreateOffenderPhoneUseCase::new(
            offender_usecase_deps.clone()
        ));
        register!(UpdateOffenderPhoneUseCase::new(
            offender_usecase_deps.clone()
        ));
        register!(DeleteOffenderPhoneUseCase::new(
            offender_usecase_deps.clone()
        ));
        register!(CreateOffenderAddressUseCase::new(
            offender_usecase_deps.clone()
        ));
        register!(UpdateOffenderAddressUseCase::new(
            offender_usecase_deps.clone()
        ));
        register!(DeleteOffenderAddressUseCase::new(
            offender_usecase_deps.clone()
        ));

        // Protective measures
        register!(CreateProtectiveMeasureUseCase::new(pm_usecase_deps.clone()));
        register!(GetProtectiveMeasureByIdUseCase::new(
            pm_usecase_deps.clone()
        ));
        register!(GetAllProtectiveMeasuresUseCase::new(
            pm_usecase_deps.clone()
        ));
        register!(GetMeasuresByVictimUseCase::new(pm_usecase_deps.clone()));
        register!(UpdateProtectiveMeasureUseCase::new(pm_usecase_deps.clone()));
        register!(DeleteProtectiveMeasureUseCase::new(pm_usecase_deps.clone()));

        // Extensions
        register!(CreateExtensionUseCase::new(extension_usecase_deps.clone()));
        register!(GetExtensionByIdUseCase::new(extension_usecase_deps.clone()));
        register!(GetExtensionsByMeasureUseCase::new(
            extension_usecase_deps.clone()
        ));
        register!(UpdateExtensionByIdUseCase::new(
            extension_usecase_deps.clone()
        ));
        register!(DeleteExtensionByIdUseCase::new(
            extension_usecase_deps.clone()
        ));

        // Attendance victims
        register!(CreateAttendanceVictimUseCase::new(
            attendance_victim_usecase_deps.clone()
        ));
        register!(GetAttendanceVictimByIdUseCase::new(
            attendance_victim_usecase_deps.clone()
        ));
        register!(GetAllAttendanceVictimsUseCase::new(
            attendance_victim_usecase_deps.clone()
        ));
        register!(GetAttendanceVictimsByVictimUseCase::new(
            attendance_victim_usecase_deps.clone()
        ));
        register!(UpdateAttendanceVictimUseCase::new(
            attendance_victim_usecase_deps.clone()
        ));
        register!(DeleteAttendanceVictimUseCase::new(
            attendance_victim_usecase_deps.clone()
        ));
        register!(GetAttendanceMembersUseCase::new(
            attendance_victim_usecase_deps.clone()
        ));
        register!(AddAttendanceMemberUseCase::new(
            attendance_victim_usecase_deps.clone()
        ));
        register!(RemoveAttendanceMemberUseCase::new(
            attendance_victim_usecase_deps.clone()
        ));

        // Attendance offenders
        register!(CreateAttendanceOffenderUseCase::new(
            attendance_offender_usecase_deps.clone()
        ));
        register!(GetAttendanceOffenderByIdUseCase::new(
            attendance_offender_usecase_deps.clone()
        ));
        register!(GetAllAttendanceOffendersUseCase::new(
            attendance_offender_usecase_deps.clone()
        ));
        register!(GetAttendanceOffendersByOffenderUseCase::new(
            attendance_offender_usecase_deps.clone()
        ));
        register!(GetAttendanceOffendersByVictimUseCase::new(
            attendance_offender_usecase_deps.clone()
        ));
        register!(UpdateAttendanceOffenderUseCase::new(
            attendance_offender_usecase_deps.clone()
        ));
        register!(DeleteAttendanceOffenderUseCase::new(
            attendance_offender_usecase_deps.clone()
        ));
        register!(GetAttendanceOffenderMembersUseCase::new(
            attendance_offender_usecase_deps.clone()
        ));
        register!(AddAttendanceOffenderMemberUseCase::new(
            attendance_offender_usecase_deps.clone()
        ));
        register!(RemoveAttendanceOffenderMemberUseCase::new(
            attendance_offender_usecase_deps.clone()
        ));

        // Presenters
        register!(WorkSessionPresenter::new(Arc::clone(
            &work_session_read_repository
        ),));
        register!(ProtectiveMeasurePresenter::new(
            Arc::clone(&extension_repository),
            Arc::clone(&victim_read_repository),
            Arc::clone(&offender_read_repository),
            Arc::clone(&city_repository),
        ));

        AppDependencies {
            password_hasher,
            config,
            usecases,
        }
    }

    pub fn configure(&self, cfg: &mut web::ServiceConfig) {
        for uc in &self.usecases {
            uc.register(cfg);
        }
        cfg.app_data(web::Data::new(self.config.clone()));
        configure_routes(cfg);
    }
}
