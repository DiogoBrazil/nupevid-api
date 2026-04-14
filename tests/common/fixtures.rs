use nupevid_api::core::commands::users::{CreateUser, UpdateUser, UpdateUserPassword};
use nupevid_api::core::value_objects::profiles::Profile;
use nupevid_api::core::value_objects::ranks::Rank;

pub fn valid_create_user() -> CreateUser {
    CreateUser {
        rank: Rank::CapPm,
        registration: "100012345".to_string(),
        full_name: "João Silva".to_string(),
        profile: Profile::Root,
        email: "joao.silva@test.com".to_string(),
        password: "senha123".to_string(),
        city_id: None,
        permission_policies: None,
    }
}

pub fn valid_create_user_2() -> CreateUser {
    CreateUser {
        rank: Rank::SdPm,
        registration: "100067890".to_string(),
        full_name: "Maria Santos".to_string(),
        profile: Profile::Root,
        email: "maria.santos@test.com".to_string(),
        password: "senha456".to_string(),
        city_id: None,
        permission_policies: None,
    }
}

pub fn create_user_with_invalid_email() -> CreateUser {
    CreateUser {
        rank: Rank::CapPm,
        registration: "100011111".to_string(),
        full_name: "Invalid User".to_string(),
        profile: Profile::Root,
        email: "invalid-email".to_string(),
        password: "senha123".to_string(),
        city_id: None,
        permission_policies: None,
    }
}

pub fn create_user_with_invalid_registration() -> CreateUser {
    CreateUser {
        rank: Rank::CapPm,
        registration: "99999".to_string(),
        full_name: "Invalid Registration User".to_string(),
        profile: Profile::Root,
        email: "invalid.reg@test.com".to_string(),
        password: "senha123".to_string(),
        city_id: None,
        permission_policies: None,
    }
}

pub fn create_user_with_empty_fields() -> CreateUser {
    CreateUser {
        rank: Rank::CapPm,
        registration: "".to_string(),
        full_name: "".to_string(),
        profile: Profile::Root,
        email: "".to_string(),
        password: "".to_string(),
        city_id: None,
        permission_policies: None,
    }
}

pub fn valid_update_user() -> UpdateUser {
    UpdateUser {
        rank: Rank::MajPm,
        registration: "100012345".to_string(),
        full_name: "João Silva Updated".to_string(),
        profile: Profile::Root,
        email: "joao.silva.updated@test.com".to_string(),
        city_id: None,
        permission_policies: None,
    }
}

pub fn valid_update_password() -> UpdateUserPassword {
    UpdateUserPassword {
        current_password: Some("senha123".to_string()),
        new_password: "novaSenha456".to_string(),
    }
}

pub fn invalid_update_password() -> UpdateUserPassword {
    UpdateUserPassword {
        current_password: Some("senhaErrada".to_string()),
        new_password: "novaSenha456".to_string(),
    }
}
