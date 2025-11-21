use nupevid_api::core::entities::users::{CreateUser, UpdateUser, UpdateUserPassword};

pub fn valid_create_user() -> CreateUser {
    CreateUser {
        rank: "Professor".to_string(),
        registration: "12345".to_string(),
        full_name: "João Silva".to_string(),
        profile: "admin".to_string(),
        email: "joao.silva@test.com".to_string(),
        password: "senha123".to_string(),
    }
}

pub fn valid_create_user_2() -> CreateUser {
    CreateUser {
        rank: "Estudante".to_string(),
        registration: "67890".to_string(),
        full_name: "Maria Santos".to_string(),
        profile: "user".to_string(),
        email: "maria.santos@test.com".to_string(),
        password: "senha456".to_string(),
    }
}

pub fn create_user_with_invalid_email() -> CreateUser {
    CreateUser {
        rank: "Professor".to_string(),
        registration: "11111".to_string(),
        full_name: "Invalid User".to_string(),
        profile: "user".to_string(),
        email: "invalid-email".to_string(),
        password: "senha123".to_string(),
    }
}

pub fn create_user_with_empty_fields() -> CreateUser {
    CreateUser {
        rank: "".to_string(),
        registration: "".to_string(),
        full_name: "".to_string(),
        profile: "".to_string(),
        email: "".to_string(),
        password: "".to_string(),
    }
}

pub fn valid_update_user() -> UpdateUser {
    UpdateUser {
        rank: "Professor Associado".to_string(),
        registration: "12345".to_string(),
        full_name: "João Silva Updated".to_string(),
        profile: "admin".to_string(),
        email: "joao.silva.updated@test.com".to_string(),
    }
}

pub fn valid_update_password() -> UpdateUserPassword {
    UpdateUserPassword {
        current_password: "senha123".to_string(),
        new_password: "novaSenha456".to_string(),
    }
}

pub fn invalid_update_password() -> UpdateUserPassword {
    UpdateUserPassword {
        current_password: "senhaErrada".to_string(),
        new_password: "novaSenha456".to_string(),
    }
}
