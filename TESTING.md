# TESTING

Guia prático da suíte de testes do `nupevid-api`. Suíte atual: **622 testes** (249 unitários + 373 de integração/repositório), todos verdes em `cargo nextest run --test-threads 1`.

## Runner

Os testes usam [`cargo-nextest`](https://nexte.st/). Instale uma vez:

```bash
cargo install cargo-nextest --locked
```

Os testes de integração e de repositório compartilham o mesmo banco PostgreSQL e **precisam** rodar sequencialmente. Sempre use `--test-threads 1`.

```bash
# Suíte completa
cargo nextest run --test-threads 1

# Só unit tests (cargo nextest só roda testes com runner; use --lib para limitar à lib)
cargo nextest run --lib

# Só integração + repositórios
cargo nextest run --test integration_tests --test-threads 1

# Filtrar por nome
cargo nextest run --test-threads 1 create_user

# Output detalhado de testes que falham
cargo nextest run --test-threads 1 --no-capture
```

`cargo test` ainda funciona (`cargo test -- --test-threads=1`), mas o output é mais pobre.

## Banco de dados de teste

Todos os testes que tocam persistência usam o `DATABASE_TEST_URL`. As migrations rodam automaticamente no primeiro setup; o helper `clean_database` limpa todas as tabelas de domínio antes de cada teste.

```bash
# .env ou export
export DATABASE_TEST_URL="postgres://postgres:postgres@localhost:5432/nupevid_test"

# Criar o banco (uma vez)
createdb nupevid_test
```

Nunca aponte `DATABASE_TEST_URL` para o banco de desenvolvimento — o `clean_database` apaga tudo.

## Estrutura

```
src/
├── usecases/<domain>/
│   ├── <usecase>_usecase.rs
│   ├── <usecase>_usecase_test.rs        # unit tests com mockall
│   └── test_support.rs                  # fakes + helpers locais do domínio
├── validators/**                        # unit tests inline (#[cfg(test)] mod tests)
├── presenters/**                        # unit tests inline
└── repositories/error_mapper.rs         # unit tests inline

tests/
├── common/                              # infra compartilhada
│   ├── fixtures.rs                      # CreateUser/UpdateUser determinísticos
│   ├── db_fixtures.rs                   # inserts diretos de city/victim/offender/etc.
│   └── test_helpers.rs                  # setup_test_db, clean_database, JWT, app
├── integration/                         # testes end-to-end via HTTP
│   └── *_test.rs
├── repositories/                        # testes do SQL direto (Fase 5)
│   └── *_repository_test.rs
└── integration_tests.rs                 # registra os módulos
```

## Camadas de teste

### 1. Unit tests de usecase (`src/usecases/<domain>/`)

Testam a lógica de negócio sem banco. Usam [`mockall`](https://docs.rs/mockall/) 0.13 para os repositórios (gerados via `#[cfg_attr(test, mockall::automock)]` nos traits em `core/contracts/repository/`) e fakes para `PasswordHasherPort`.

Cada domínio tem um `test_support.rs` com fakes e construtores reutilizáveis. Para `users/`:

```rust
use crate::usecases::users::test_support::{claims, deps, empty_policies, FakePasswordHasher, user_record};
use crate::core::contracts::repository::users::MockUserRepository;

#[actix_web::test]
async fn create_user_success() {
    let mut repo = MockUserRepository::new();
    repo.expect_check_user_exists_by_email()
        .returning(|_| Ok(false));
    repo.expect_check_user_exists_by_registration()
        .returning(|_| Ok(false));
    repo.expect_create_user()
        .returning(|u| Ok(user_record(u.id, u.profile, u.city_id, empty_policies())));

    let deps = deps(repo, FakePasswordHasher::ok());
    let result = CreateUserUseCase::execute(
        &deps,
        claims(Profile::Root, Uuid::new_v4(), None),
        valid_payload(),
    ).await;

    assert!(result.is_ok());
}
```

Execute apenas os unit tests de um usecase:

```bash
cargo nextest run --lib usecases::users::create_user_usecase_test
```

### 2. Testes de integração (`tests/integration/`)

Testam fluxos completos via HTTP passando pelo `AuthMiddleware`. Usam o `AppDependencies` real montado em cima do pool de teste.

```rust
use crate::common::test_helpers::{
    build_root_claims, clean_database, create_full_test_app,
    build_test_config, generate_jwt, setup_test_db, with_auth_headers,
};

#[actix_web::test]
async fn create_user_returns_201() {
    let pool = setup_test_db().await;
    clean_database(&pool).await;
    let config = build_test_config();
    let app = create_full_test_app(pool.clone(), config.clone()).await;

    let token = generate_jwt(&build_root_claims(), &config.jwt_secret);
    let req = with_auth_headers(
        test::TestRequest::post().uri("/api/v1/users"),
        &config,
        &token,
    )
    .set_json(&fixtures::valid_create_user())
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
}
```

### 3. Testes de repositório (`tests/repositories/`)

Chamam os repositórios Postgres **diretamente** (sem HTTP, sem usecase). Cobrem SQL crítico: índices parciais, soft-delete, filtros por `allowed_cities`, joins, `DuplicateEntry` vs `NotFound`. Cada arquivo foca em um repositório (`protective_measures`, `work_sessions`, `attendance_members`, `users`).

## Helpers principais

`tests/common/test_helpers.rs`:

| Helper | Uso |
|---|---|
| `setup_test_db()` | Conecta ao `DATABASE_TEST_URL` e roda migrations. |
| `clean_database(&pool)` | Limpa todas as tabelas de domínio em ordem FK-safe. |
| `build_test_config()` | `Config` com defaults seguros para teste. |
| `create_full_test_app(pool, config)` | App real com `AuthMiddleware` e rotas `/api/v1`. |
| `build_root_claims()` / `build_city_admin_claims(city)` / `build_city_user_claims(city)` | Claims JWT por perfil. |
| `generate_jwt(&claims, &secret)` | Assina o token HS256. |
| `with_auth_headers(req, &config, token)` | Adiciona `api_key` + `Authorization: Bearer …`. |
| `create_work_session_for_user(pool, user_id)` | Insere uma sessão ativa com o usuário como Commander. |

`tests/common/fixtures.rs` — payloads determinísticos para `CreateUser` / `UpdateUser` / `UpdateUserPassword`.

`tests/common/db_fixtures.rs` — inserts diretos de entidades auxiliares (`insert_city`, `insert_victim`, `insert_offender`, …) quando o setup do teste precisa atalhar o fluxo HTTP.

## Escrevendo um novo teste

**Unit test de usecase**
1. Crie `src/usecases/<domain>/<usecase>_usecase_test.rs`.
2. Registre em `src/usecases/<domain>/mod.rs` sob `#[cfg(test)] mod <usecase>_usecase_test;`.
3. Reaproveite o `test_support.rs` do domínio; adicione fakes novos ali se precisar em múltiplos testes.

**Integração**
1. Crie `tests/integration/<feature>_test.rs`.
2. Registre em `tests/integration/mod.rs` (`pub mod <feature>_test;`).
3. Sempre comece com `setup_test_db` + `clean_database`.

**Repositório**
1. Crie `tests/repositories/<repo>_repository_test.rs`.
2. Registre em `tests/repositories/mod.rs`.

## Paralelismo

Não rode testes de integração/repositório em paralelo. Todos compartilham o mesmo schema e o `clean_database` é global. Unit tests (`--lib`) podem rodar em paralelo sem problema, mas o padrão do projeto é sempre `--test-threads 1` para uniformidade.

## CI

Para validar localmente antes de commit:

```bash
cargo check --tests
cargo clippy --all-targets --no-deps
cargo nextest run --test-threads 1
```

## Troubleshooting

- **`Failed to run migrations`** — o banco existe mas o schema está inconsistente. `dropdb nupevid_test && createdb nupevid_test` e rode de novo.
- **Teste que passa isolado mas falha em suíte** — quase sempre é um teste anterior deixando estado; garanta que começa com `clean_database`.
- **`DuplicateEntry` em teste de criação** — provavelmente o fixture é determinístico e já existe no banco; use `clean_database` ou um `db_fixtures::insert_*` com UUID novo.
- **`cargo nextest` não encontrado** — `cargo install cargo-nextest --locked`.
