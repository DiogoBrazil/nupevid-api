# TESTING

Guia da suíte de testes do `nupevid-api` após migração para `#[sqlx::test]` nos testes que usam banco.

## Runner padrão

Use o wrapper oficial do projeto:

```bash
./test.sh
```

Ele faz:

1. Carrega variáveis de `.env` (se existir).
2. Usa `DATABASE_TEST_URL` como base para `DATABASE_URL` do `sqlx::test`.
3. Executa `cargo nextest run` com os argumentos passados.

Também aceita filtros do `nextest`:

```bash
./test.sh --test integration_tests
./test.sh attendance_filter_by_measure
./test.sh --no-capture
```

## Banco de teste e `sqlx::test`

Testes de integração/repositório com banco agora usam `#[sqlx::test]` e recebem `pool: sqlx::PgPool` injetado.

- Requisito: usuário de `DATABASE_TEST_URL` com permissão `CREATEDB`.
- Paralelismo: seguro por padrão; cada teste usa banco efêmero isolado.
- Em ambientes com `max_connections` baixo, ajuste `--test-threads` se necessário.

- Não usar mais `setup_test_db()`.
- Não usar mais `clean_database()`.
- O isolamento do banco por teste é gerenciado pelo `sqlx::test`.

Configure apenas:

```bash
export DATABASE_TEST_URL="postgres://postgres:postgres@localhost:5432/nupevid_test"
createdb nupevid_test
```

Nunca aponte para banco de desenvolvimento.

## Padrão para novos testes de integração

```rust
#[sqlx::test]
async fn create_user_returns_201(pool: sqlx::PgPool) {
    let config = test_helpers::build_test_config();
    let app = test_helpers::create_full_test_app(pool.clone(), config.clone()).await;

    let token = test_helpers::generate_jwt(&test_helpers::build_root_claims(), &config.jwt_secret);
    let req = test_helpers::with_auth_headers(
        test::TestRequest::post().uri("/api/v1/users"),
        &config,
        &token,
    )
    .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
}
```

Para testes sem banco (ex.: validações puras de estrutura/matriz), prefira `#[test]` síncrono.

## Helpers principais

`tests/common/test_helpers.rs` mantém:

- `build_test_config()` (com `database_url` placeholder vazio).
- `create_full_test_app(pool, config)`.
- helpers de JWT (`build_*_claims`, `generate_jwt`, `with_auth_headers`).
- `create_work_session_for_user(pool, user_id)`.

## Comandos úteis

```bash
cargo check --tests
./test.sh
./test.sh --test integration_tests
./test.sh --test-threads 4
```

## Troubleshooting

- `DATABASE_TEST_URL is not set` → configure no `.env` ou no ambiente.
- `cargo nextest` ausente → `cargo install cargo-nextest --locked`.
- Falha de conexão com banco → confirme host/porta/credenciais e se o banco existe.
